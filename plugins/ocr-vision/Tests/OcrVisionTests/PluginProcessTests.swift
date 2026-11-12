// 起插件的可执行文件，在管道上说 JSON-RPC，和宿主一样：环境先清空，暂存目录照协议 §7.1 摆。
// 每一行 stdout 都得是 JSON-RPC 2.0。清单的版本、id、命令和代码对得上。
import Foundation
import Testing

@testable import TransformLoop
@testable import VisionOCR

/// `swift test` 编出来的插件可执行文件，在测试包旁边。
private func executable() -> URL {
    Bundle(for: Lines.self).bundleURL.deletingLastPathComponent().appending(path: "usuyo-plugin-ocr-vision")
}

@Suite(.timeLimit(.minutes(2))) struct PluginProcess {
    @Test func describeHealthRunAndExit() async throws {
        let call = try StagedCall(call: "1")
        defer { call.remove() }
        let home = call.root.appending(path: "home")
        let tmp = call.root.appending(path: "tmp")
        try FileManager.default.createDirectory(at: home, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: tmp, withIntermediateDirectories: true)
        let card = try call.stage(samples.appending(path: "card.png"), index: 0)

        let exe = executable()
        let p = Process()
        p.executableURL = exe
        p.environment = [
            "USUYO_PROTOCOL": "1", "USUYO_PLUGIN_DIR": exe.deletingLastPathComponent().path(percentEncoded: false),
            "HOME": home.path(percentEncoded: false), "TMPDIR": tmp.path(percentEncoded: false), "PATH": "/usr/bin:/bin",
        ]
        p.currentDirectoryURL = tmp
        let stdin = Pipe()
        let stdout = Pipe()
        p.standardInput = stdin
        p.standardOutput = stdout
        try p.run()
        let lines = Lines(fd: stdout.fileHandleForReading.fileDescriptor)
        func send(_ m: JSON) throws {
            writeAll(stdin.fileHandleForWriting.fileDescriptor, [UInt8](try m.line()) + [0x0A])
        }
        func next() async throws -> JSON {
            let m = try await lines.next()
            #expect(m["jsonrpc"] == .string("2.0"), "\(m)")
            return m
        }

        try send(.object(["jsonrpc": .string("2.0"), "id": .int(1), "method": .string("describe")]))
        let d = try await next()
        #expect(d["result"]?["protocol"] == .int(1))
        #expect(d["result"]?["version"] == .string(pluginVersion))
        guard case .array(let models)? = d["result"]?["models"] else {
            Issue.record("no models in \(d)")
            return
        }
        #expect(models.contains(.string("zh-Hans")) && models.contains(.string("en-US")))

        try send(.object(["jsonrpc": .string("2.0"), "id": .int(2), "method": .string("health")]))
        #expect(try await next()["result"]?["ready"] == .bool(true))

        try send(runMessage(id: .int(3), params: runParams(
            inputs: [inputJSON(kind: "image", path: card, imageKey: "3f2a9c0d1e4b5a67")], out: call.out.path(percentEncoded: false))))
        var progress = 0
        var reply = try await next()
        while reply["method"] == .string("progress") {
            #expect(reply["params"]?["id"] == .int(3))
            progress += 1
            reply = try await next()
        }
        #expect(progress == 1)
        #expect(reply["id"] == .int(3))
        #expect(reply["result"]?["outputs"] == .array([.object(["kind": .string("ocr"), "path": .string("ocr.json")])]))
        let doc = try JSON.parse(Data(contentsOf: call.out.appending(path: "ocr.json")))
        if case .array(let images)? = doc["images"], case .array(let found)? = images.first?["lines"] {
            #expect(found.count == 4)
        } else {
            Issue.record("unexpected ocr.json: \(doc)")
        }

        // 坏的一行不让进程退出
        writeAll(stdin.fileHandleForWriting.fileDescriptor, Array("{oops\n".utf8))
        #expect(try await next()["error"]?["code"] == .int(-32700))

        try stdin.fileHandleForWriting.close()
        while p.isRunning { try await Task.sleep(for: .milliseconds(10)) }
        #expect(p.terminationStatus == 0)
    }

    @Test func manifestMatchesTheCode() throws {
        let text = try String(contentsOf: repo.appending(path: "plugins/ocr-vision/usuyo-plugin.yaml"), encoding: .utf8)
        let lines = Set(text.split(separator: "\n").map(String.init))
        for expected in [
            "id: builtin/ocr-vision",
            "kind: transform",
            "version: \(pluginVersion)",
            "license: MIT",
            "consumes: [image, derived:keyframes]",
            "produces: [ocr]",
            "provider: none",
            #"  command: ["${PLUGIN_DIR}/usuyo-plugin-ocr-vision"]"#,
        ] {
            #expect(lines.contains(expected), "\(expected)")
        }
        #expect(!text.contains("network"))
    }
}
