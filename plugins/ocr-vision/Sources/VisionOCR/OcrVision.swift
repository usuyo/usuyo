import Foundation
import TransformLoop
import Vision

/// 插件版本；宿主拿它和清单的 `version` 比，对不上不加载。
public let pluginVersion = "0.1.0"

/// 产物的文件名，相对 `out`。
let outputFile = "ocr.json"

/// `builtin/ocr-vision` 的 `run` 和 `health`（协议 §4.2）。
///
/// 一次 `run`：查 `want`、参数、`out` 和每个输入 → 一张一张读图、认字，图之间查取消、发 `progress`
/// → 所有图都没有一行分数到 0.3 时回 `empty` / `no-text`，否则写一份盖住所有输入的 `ocr.json`。
public struct OcrVision: Transform {
    public init() {}

    /// `describe` 的回答：`models` 是 accurate 一档认得的语言。
    public func describe() -> Describe {
        Describe(version: pluginVersion, models: (try? supportedLanguages(.accurate)) ?? [])
    }

    public func health() -> Health {
        let ok = VNRecognizeTextRequest.supportedRevisions.contains(requestRevision)
        return Health(
            ready: ok,
            checks: [
                HealthCheck(
                    name: "vision:text-revision-\(requestRevision)", ok: ok,
                    message: ok ? nil : "this macOS has no VNRecognizeTextRequest revision \(requestRevision)")
            ])
    }

    public func run(_ params: RunParams, _ context: RunContext) async throws(CallFailure) -> RunResult {
        guard params.want == [.ocr] else {
            throw CallFailure(.invalidArgs, "this plugin only produces `ocr`")
        }
        guard !params.inputs.isEmpty else {
            throw CallFailure(.invalidArgs, "no inputs")
        }
        let options = try OcrOptions(params.options, supported: supportedLanguages)
        let dirs = try CallDirs(out: params.out)
        let files = try params.inputs.enumerated().map { (i, input) throws(CallFailure) in
            try check(input, index: i)
            return try dirs.input(input.path, index: i)
        }

        var images: [OcrImage] = []
        for (i, input) in params.inputs.enumerated() {
            try context.checkCancelled()
            let file = files[i]
            let read = await onOwnThread { () -> (LoadedImage, Result<[OcrLine], any Error>)? in
                guard let loaded = loadImage(file) else { return nil }
                return (loaded, Result { try recognize(loaded, options) })
            }
            guard let (loaded, recognized) = read else {
                throw CallFailure(.unsupportedInput, "input \(i) is not a readable image")
            }
            let lines: [OcrLine]
            switch recognized {
            case .success(let l): lines = l
            case .failure(let error):
                throw CallFailure(.internal, "input \(i): Vision failed: \(error.localizedDescription)")
            }
            images.append(
                OcrImage(
                    input: i, imageKey: input.imageKey, frameSec: input.frameSec,
                    width: loaded.width, height: loaded.height, lines: lines))
            await context.progress(done: Double(i + 1), total: Double(params.inputs.count), unit: "image")
        }
        try context.checkCancelled()

        let document = OcrDocument(images: images)
        var result = RunResult(outputs: [], tool: visionTool(), options: options.echoed)
        guard document.hasText else {
            result.empty = true
            result.reason = .noText
            result.marks = [Mark(code: .noText)]
            return result
        }
        do {
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
            // 不覆盖、不跟着已经在的符号链接写。
            try encoder.encode(document).write(to: dirs.out.appending(path: outputFile), options: .withoutOverwriting)
        } catch {
            throw CallFailure(.internal, "write \(outputFile): \(error.localizedDescription)")
        }
        result.outputs = [RunProduct(kind: .ocr, path: outputFile)]
        return result
    }
}

/// 输入的种类和它带的定位：图文的图、封面是 `image` 带 `imageKey`，关键帧是 `derived:keyframes` 带 `frameSec`。
private func check(_ input: RunInput, index i: Int) throws(CallFailure) {
    switch input.kind {
    case .image:
        guard let key = input.imageKey, input.frameSec == nil else {
            throw CallFailure(.invalidArgs, "input \(i): an `image` input needs `imageKey` and no `frameSec`")
        }
        guard key.count == 16, key.allSatisfy({ "0123456789abcdef".contains($0) }) else {
            throw CallFailure(.invalidArgs, "input \(i): `imageKey` must be 16 lowercase hex digits")
        }
    case .derivedKeyframes:
        guard let sec = input.frameSec, input.imageKey == nil else {
            throw CallFailure(.invalidArgs, "input \(i): a `derived:keyframes` input needs `frameSec` and no `imageKey`")
        }
        guard sec.isFinite, sec >= 0 else {
            throw CallFailure(.invalidArgs, "input \(i): `frameSec` must be a finite number of seconds, not below 0")
        }
    default:
        throw CallFailure(.invalidArgs, "input \(i): this plugin reads `image` and `derived:keyframes`, not `\(input.kind.rawValue)`")
    }
}

/// 在一个自己的线程上跑阻塞的活（解码、Vision 的 perform，一张图零点几到几秒），不占 Swift 并发的线程池。
private func onOwnThread<T: Sendable>(_ work: @escaping @Sendable () -> T) async -> T {
    await withCheckedContinuation { done in
        let thread = Thread { done.resume(returning: work()) }
        thread.name = "usuyo-ocr-vision-image"
        thread.start()
    }
}
