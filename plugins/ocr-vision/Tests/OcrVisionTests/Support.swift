import CoreGraphics
import Foundation
import ImageIO
import Testing

@testable import TransformLoop

/// 等一条消息最多多久；测试里没有哪一步该这么慢（Vision 第一次加载约 2 秒）。
let wait: Duration = .seconds(30)

/// 仓库根。
let repo = URL(filePath: #filePath)
    .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
    .deletingLastPathComponent().deletingLastPathComponent()

/// 003 的转换样例（crates/usuyo-core/testdata/transform/）。
let samples = repo.appending(path: "crates/usuyo-core/testdata/transform")

/// 从一个文件描述符按行读 JSON，每次读都有时限。
final class Lines {
    let fd: Int32
    private var buffer: [UInt8] = []

    init(fd: Int32) {
        self.fd = fd
    }

    /// 下一行；时限内没有完整的一行、或对面关了时测试失败。不阻塞并发线程池：没有数据时睡 5 毫秒再看。
    func next(sourceLocation: SourceLocation = #_sourceLocation) async throws -> JSON {
        let deadline = ContinuousClock.now + wait
        while true {
            if let i = buffer.firstIndex(of: UInt8(ascii: "\n")) {
                let line = Array(buffer[..<i])
                buffer.removeSubrange(...i)
                return try JSON.parse(line)
            }
            var p = pollfd(fd: fd, events: Int16(POLLIN), revents: 0)
            guard poll(&p, 1, 0) > 0 else {
                guard ContinuousClock.now < deadline else {
                    Issue.record("no line within \(wait)", sourceLocation: sourceLocation)
                    throw Timeout()
                }
                try await Task.sleep(for: .milliseconds(5))
                continue
            }
            var chunk = [UInt8](repeating: 0, count: 64 * 1024)
            let n = chunk.withUnsafeMutableBytes { read(fd, $0.baseAddress, $0.count) }
            guard n > 0 else {
                Issue.record("the other end closed", sourceLocation: sourceLocation)
                throw Timeout()
            }
            buffer.append(contentsOf: chunk[..<n])
        }
    }

    /// 下一条带 `id` 的（回答），跳过 `progress`。
    func reply(sourceLocation: SourceLocation = #_sourceLocation) async throws -> JSON {
        while true {
            let m = try await next(sourceLocation: sourceLocation)
            if m["id"] != nil { return m }
        }
    }

    struct Timeout: Error {}
}

/// 写一整段字节到文件描述符。
func writeAll(_ fd: Int32, _ bytes: [UInt8]) {
    var offset = 0
    while offset < bytes.count {
        let n = bytes.withUnsafeBytes { write(fd, $0.baseAddress.map { $0 + offset }, $0.count - offset) }
        if n <= 0 { return }
        offset += n
    }
}

/// 在内存里的管道上跑循环：一端当宿主写请求，另一端读回答。
final class LoopHost {
    private let toPlugin = Pipe()
    private let fromPlugin = Pipe()
    private var writeOpen = true
    let lines: Lines
    let serving: Task<Void, any Error>

    init(_ transform: some Transform) {
        let input = toPlugin.fileHandleForReading.fileDescriptor
        let output = fromPlugin.fileHandleForWriting.fileDescriptor
        lines = Lines(fd: fromPlugin.fileHandleForReading.fileDescriptor)
        serving = Task {
            try await serve(
                describe: Describe(version: "0.1.0", models: ["zh-Hans"]), transform: transform,
                input: input, output: output)
        }
    }

    func sendRaw(_ bytes: [UInt8]) {
        writeAll(toPlugin.fileHandleForWriting.fileDescriptor, bytes)
    }

    func send(_ message: JSON) throws {
        sendRaw([UInt8](try message.line()) + [UInt8(ascii: "\n")])
    }

    /// 关掉插件的 stdin。
    func closeInput() throws {
        if writeOpen {
            writeOpen = false
            try toPlugin.fileHandleForWriting.close()
        }
    }

    /// 关掉 stdin，等循环返回。
    func close() async throws -> Result<Void, any Error> {
        try closeInput()
        return await serving.result
    }
}

/// 一个用闭包写的转换插件。
struct ClosureTransform: Transform {
    let body: @Sendable (RunParams, RunContext) async throws(CallFailure) -> RunResult

    func run(_ params: RunParams, _ context: RunContext) async throws(CallFailure) -> RunResult {
        try await body(params, context)
    }

    func health() -> Health {
        Health(ready: true, checks: [])
    }
}

/// 照协议 §7.1 摆的一次调用：`<root>/in/<call>/<下标>/<文件>` 和 `<root>/out/<call>/`。
struct StagedCall {
    let root: URL
    let call: String
    var out: URL { root.appending(path: "out/\(call)") }

    init(call: String = "7") throws {
        root = FileManager.default.temporaryDirectory.appending(path: "ocr-vision-test-\(UUID().uuidString)")
        self.call = call
        try FileManager.default.createDirectory(at: root.appending(path: "in/\(call)"), withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: out, withIntermediateDirectories: true)
    }

    /// 把 `source` 复制成第 `index` 个输入，回它在 `in/` 里的路径。
    func stage(_ source: URL, index: Int) throws -> String {
        let dir = root.appending(path: "in/\(call)/\(index)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let dest = dir.appending(path: source.lastPathComponent)
        try FileManager.default.copyItem(at: source, to: dest)
        return dest.path(percentEncoded: false)
    }

    /// 测试自己建的临时目录，测完删掉。
    func remove() {
        try? FileManager.default.removeItem(at: root)
    }
}

/// 画一张纯色、没有字的 PNG。
func blankPNG(at url: URL, width: Int = 640, height: Int = 480) throws {
    let space = try #require(CGColorSpace(name: CGColorSpace.sRGB))
    let ctx = try #require(
        CGContext(
            data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: 0, space: space,
            bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue))
    ctx.setFillColor(red: 1, green: 1, blue: 1, alpha: 1)
    ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
    let image = try #require(ctx.makeImage())
    let dest = try #require(CGImageDestinationCreateWithURL(url as CFURL, "public.png" as CFString, 1, nil))
    CGImageDestinationAddImage(dest, image, nil)
    #expect(CGImageDestinationFinalize(dest))
}

/// `run` 请求。
func runMessage(id: JSON, params: JSON) -> JSON {
    .object(["jsonrpc": .string("2.0"), "id": id, "method": .string("run"), "params": params])
}

/// 一个输入的 JSON。
func inputJSON(kind: String, path: String, imageKey: String? = nil, frameSec: Double? = nil) -> JSON {
    var o: [String: JSON] = ["kind": .string(kind), "path": .string(path)]
    if let imageKey { o["imageKey"] = .string(imageKey) }
    if let frameSec { o["frameSec"] = .double(frameSec) }
    return .object(o)
}

func runParams(inputs: [JSON], out: String, options: [String: JSON] = [:], want: [String] = ["ocr"]) -> JSON {
    .object([
        "item": .object(["id": .string("fixture-local-0001"), "kind": .string("post"), "title": .string("样例")]),
        "want": .array(want.map(JSON.string)),
        "inputs": .array(inputs),
        "out": .string(out),
        "options": .object(options),
        "provider": .null,
    ])
}

/// 插件自己失败的回答（`-32000`）里的错误码。
func failureCode(_ reply: JSON, sourceLocation: SourceLocation = #_sourceLocation) -> String? {
    #expect(reply["error"]?["code"] == .int(-32000), "\(reply)", sourceLocation: sourceLocation)
    return reply["error"]?["data"]?["code"]?.string
}
