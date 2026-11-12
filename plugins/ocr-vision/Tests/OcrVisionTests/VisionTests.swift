// 真的调 Vision：样例 card.png 的四行、空白图的 no-text、一次两个输入、读不了的图和越界的路径。
import Foundation
import Testing

@testable import TransformLoop
@testable import VisionOCR

/// 直接调处理函数用的 `RunContext`；`progress` 写进 /dev/null。
private func context() -> RunContext {
    let null = FileHandle.nullDevice.fileDescriptor
    return RunContext(
        id: .number(1),
        session: Session(describe: Describe(version: pluginVersion, models: []), transform: OcrVision(), sink: Sink(fd: null)))
}

private func params(_ call: StagedCall, _ inputs: [RunInput], options: [String: JSON] = [:]) -> RunParams {
    RunParams(
        item: RunItem(id: "fixture-local-0001", kind: "post", title: "样例"), want: [.ocr], inputs: inputs,
        out: call.out.path(percentEncoded: false), options: options)
}

private func readDocument(_ call: StagedCall) throws -> JSON {
    try JSON.parse(Data(contentsOf: call.out.appending(path: "ocr.json")))
}

/// card.swift 画的四行（crates/usuyo-core/testdata/transform/README.md）。
private let cardLines = ["手冲咖啡三段式注水", "Pour over in 3 stages", "水温 92°C，粉水比 1:15", "闷蒸 30 秒 Bloom"]

/// Vision 认中文时把汉字和数字之间的空格去掉（「粉水比1:15」），比的时候不看空白。
private func squeezed(_ s: String) -> String {
    s.filter { !$0.isWhitespace }
}

private func failure(_ body: () async throws(CallFailure) -> RunResult) async -> CallFailure? {
    do {
        _ = try await body()
        return nil
    } catch {
        return error
    }
}

@Suite(.timeLimit(.minutes(2))) struct VisionRuns {
    @Test func cardYieldsItsFourLinesTopToBottom() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let path = try call.stage(samples.appending(path: "card.png"), index: 0)
        let result = try await OcrVision().run(
            params(call, [RunInput(kind: .image, path: path, imageKey: "3f2a9c0d1e4b5a67")]), context())
        #expect(!result.empty && result.reason == nil)
        #expect(result.outputs.map(\.path) == ["ocr.json"])
        #expect(result.tool.name == "Vision")
        #expect(result.tool.version.contains("revision 3"))
        #expect(result.options["level"] == .string("accurate"))
        #expect(result.options["languages"] == .array(OcrOptions.defaultLanguages.map(JSON.string)))
        #expect(result.options["languageCorrection"] == .bool(true))

        let doc = try readDocument(call)
        guard case .array(let images)? = doc["images"], images.count == 1, case .array(let lines)? = images[0]["lines"] else {
            Issue.record("unexpected ocr.json: \(doc)")
            return
        }
        #expect(images[0]["input"] == .int(0))
        #expect(images[0]["imageKey"] == .string("3f2a9c0d1e4b5a67"))
        #expect(images[0]["frameSec"] == nil)
        #expect(images[0]["width"] == .int(1080) && images[0]["height"] == .int(1440))
        let texts = lines.compactMap { $0["text"]?.string }
        #expect(texts.map(squeezed) == cardLines.map(squeezed))
        #expect(texts.contains("Pour over in 3 stages"))
        let ys = lines.compactMap { line -> Double? in
            guard case .array(let b)? = line["bbox"], b.count == 4, case .double(let y) = b[1] else { return nil }
            return y
        }
        #expect(ys.count == 4)
        #expect(ys == ys.sorted())
        #expect((ys.first ?? 1) < (ys.last ?? 0))
        for line in lines {
            guard case .array(let b)? = line["bbox"] else { continue }
            for v in b {
                let d: Double = if case .double(let x) = v { x } else if case .int(let i) = v { Double(i) } else { -1 }
                #expect((0...1).contains(d), "\(line)")
            }
        }
    }

    @Test func blankImageIsNoText() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let blank = call.root.appending(path: "blank.png")
        try blankPNG(at: blank)
        let path = try call.stage(blank, index: 0)
        let result = try await OcrVision().run(params(call, [RunInput(kind: .derivedKeyframes, path: path, frameSec: 0.5)]), context())
        #expect(result.empty)
        #expect(result.reason == .noText)
        #expect(result.outputs.isEmpty)
        #expect(result.marks.map(\.code) == [.noText])
        #expect(!FileManager.default.fileExists(atPath: call.out.appending(path: "ocr.json").path(percentEncoded: false)))
    }

    @Test func twoInputsGiveTwoEntries() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let blank = call.root.appending(path: "frame.png")
        try blankPNG(at: blank, width: 960, height: 540)
        let first = try call.stage(samples.appending(path: "card.png"), index: 0)
        let second = try call.stage(blank, index: 1)
        let result = try await OcrVision().run(
            params(call, [
                RunInput(kind: .image, path: first, imageKey: "3f2a9c0d1e4b5a67"),
                RunInput(kind: .derivedKeyframes, path: second, frameSec: 12.04),
            ]), context())
        #expect(!result.empty)
        guard case .array(let images)? = try readDocument(call)["images"], images.count == 2 else {
            Issue.record("expected two images")
            return
        }
        #expect(images[0]["input"] == .int(0) && images[0]["imageKey"] == .string("3f2a9c0d1e4b5a67"))
        #expect(images[1]["input"] == .int(1) && images[1]["frameSec"] == .double(12.04) && images[1]["imageKey"] == nil)
        #expect(images[1]["width"] == .int(960) && images[1]["height"] == .int(540))
        #expect(images[1]["lines"] == .array([]))
    }

    @Test func unreadableImageNamesTheIndexNotThePath() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let bogus = call.root.appending(path: "bogus.png")
        try Data("not an image".utf8).write(to: bogus)
        let good = try call.stage(samples.appending(path: "card.png"), index: 0)
        let bad = try call.stage(bogus, index: 1)
        let f = await failure { () async throws(CallFailure) in
            try await OcrVision().run(
                params(call, [
                    RunInput(kind: .image, path: good, imageKey: "3f2a9c0d1e4b5a67"),
                    RunInput(kind: .image, path: bad, imageKey: "0000000000000001"),
                ]), context())
        }
        #expect(f?.code == .unsupportedInput)
        #expect(f?.message == "input 1 is not a readable image")
        #expect(!FileManager.default.fileExists(atPath: call.out.appending(path: "ocr.json").path(percentEncoded: false)))
    }

    @Test func pathsOutsideTheCallAreRefused() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let card = samples.appending(path: "card.png")
        // 直接给资料库里的原文件，没放进 in/<callId>/
        var f = await failure { () async throws(CallFailure) in
            try await OcrVision().run(params(call, [RunInput(kind: .image, path: card.path(percentEncoded: false), imageKey: "3f2a9c0d1e4b5a67")]), context())
        }
        #expect(f?.code == .invalidArgs)
        #expect(f?.message.contains("input 0") == true)
        #expect(f?.message.contains(card.path(percentEncoded: false)) == false)

        // in/ 里一个指向外面的符号链接
        let dir = call.root.appending(path: "in/\(call.call)/0")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let link = dir.appending(path: "card.png")
        try FileManager.default.createSymbolicLink(at: link, withDestinationURL: card)
        f = await failure { () async throws(CallFailure) in
            try await OcrVision().run(params(call, [RunInput(kind: .image, path: link.path(percentEncoded: false), imageKey: "3f2a9c0d1e4b5a67")]), context())
        }
        #expect(f?.code == .invalidArgs)

        // 带 `..` 的路径
        let staged = try call.stage(card, index: 1)
        let dotted = call.root.appending(path: "in/\(call.call)/1").path(percentEncoded: false) + "/../1/card.png"
        f = await failure { () async throws(CallFailure) in
            try await OcrVision().run(params(call, [RunInput(kind: .image, path: dotted, imageKey: "3f2a9c0d1e4b5a67")]), context())
        }
        #expect(f?.code == .invalidArgs)

        // `out` 不是 `<暂存>/out/<callId>`
        f = await failure { () async throws(CallFailure) in
            var p = params(call, [RunInput(kind: .image, path: staged, imageKey: "3f2a9c0d1e4b5a67")])
            p.out = call.root.path(percentEncoded: false)
            return try await OcrVision().run(p, context())
        }
        #expect(f?.code == .invalidArgs)
    }

    @Test func wrongKindsAndLocatorsAreRefused() async throws {
        let call = try StagedCall()
        defer { call.remove() }
        let path = try call.stage(samples.appending(path: "card.png"), index: 0)
        for input in [
            RunInput(kind: .mediaVideo, path: path),
            RunInput(kind: .image, path: path),
            RunInput(kind: .image, path: path, imageKey: "3F2A9C0D1E4B5A67"),
            RunInput(kind: .image, path: path, imageKey: "3f2a9c0d1e4b5a67", frameSec: 1),
            RunInput(kind: .derivedKeyframes, path: path, frameSec: -1),
        ] {
            let f = await failure { () async throws(CallFailure) in try await OcrVision().run(params(call, [input]), context()) }
            #expect(f?.code == .invalidArgs, "\(input)")
        }
        let f = await failure { () async throws(CallFailure) in
            try await OcrVision().run(params(call, [RunInput(kind: .image, path: path, imageKey: "3f2a9c0d1e4b5a67")], options: ["threads": .int(2)]), context())
        }
        #expect(f?.code == .invalidArgs)
    }
}
