// 用内存里的管道驱动循环，照 plugin-kit 的 transform/tests.rs 一条条对：describe、run、cancel、
// 坏的和太长的帧、EOF。不调 Vision。
import Foundation
import Testing

@testable import TransformLoop

private func params(_ options: [String: JSON] = ["vadThreshold": .double(0.4)]) -> JSON {
    runParams(inputs: [inputJSON(kind: "image", path: "/tmp/p/in/1/0/a.png", imageKey: "3f2a9c0d1e4b5a67")], out: "/tmp/p/out/1", options: options)
}

private func run(_ id: JSON, _ options: [String: JSON] = ["vadThreshold": .double(0.4)]) -> JSON {
    runMessage(id: id, params: params(options))
}

private func cancel(_ id: JSON) -> JSON {
    .object(["jsonrpc": .string("2.0"), "method": .string("cancel"), "params": .object(["id": id])])
}

private func request(_ id: JSON, _ method: String) -> JSON {
    .object(["jsonrpc": .string("2.0"), "id": id, "method": .string(method)])
}

private func echoResult(_ p: RunParams) -> RunResult {
    var options = p.options
    options["level"] = .string("accurate")
    return RunResult(
        outputs: [RunProduct(kind: .ocr, path: "ocr.json")], tool: UpstreamVersion(name: "Vision", version: "test"),
        options: options)
}

private let echo = ClosureTransform { p, _ in echoResult(p) }

/// 一直跑到被取消，每 20 毫秒发一次 `progress`。
private let untilCancelled = ClosureTransform { _, ctx throws(CallFailure) in
    var done = 0.0
    let deadline = ContinuousClock.now + wait
    while ContinuousClock.now < deadline {
        try ctx.checkCancelled()
        await ctx.progress(done: done, total: 100, unit: "%")
        done += 1
        try? await Task.sleep(for: .milliseconds(20))
    }
    throw CallFailure(.internal, "never cancelled")
}

@Suite(.timeLimit(.minutes(1))) struct Loop {
    @Test func describeHealthRunResult() async throws {
        let host = LoopHost(echo)
        try host.send(request(.int(1), "describe"))
        let d = try await host.lines.next()
        #expect(d["id"] == .int(1))
        #expect(d["result"] == .object(["protocol": .int(1), "version": .string("0.1.0"), "models": .array([.string("zh-Hans")])]))
        try host.send(.object(["jsonrpc": .string("2.0"), "id": .string("h"), "method": .string("health"), "params": .object([:])]))
        #expect(try await host.lines.next() == .object(["jsonrpc": .string("2.0"), "id": .string("h"), "result": .object(["ready": .bool(true), "checks": .array([])])]))
        try host.send(run(.int(7)))
        let r = try await host.lines.next()
        #expect(r["id"] == .int(7))
        #expect(r["result"]?["outputs"] == .array([.object(["kind": .string("ocr"), "path": .string("ocr.json")])]))
        #expect(r["result"]?["options"] == .object(["vadThreshold": .double(0.4), "level": .string("accurate")]))
        #expect(r["result"]?["empty"] == .bool(false))
        #expect(r["result"]?["marks"] == .array([]))
        #expect(r["result"]?["reason"] == nil)
        // 宿主收到回答马上发下一个：不算正在跑
        try host.send(run(.int(8)))
        #expect(try await host.lines.next()["result"] != nil)
        let end = try await host.close()
        #expect(throws: Never.self) { try end.get() }
    }

    @Test func cancelDuringARun() async throws {
        let host = LoopHost(untilCancelled)
        try host.send(run(.int(3)))
        let progress = try await host.lines.next()
        #expect(progress["method"] == .string("progress"))
        #expect(progress["params"]?["id"] == .int(3))
        #expect(progress["id"] == nil, "progress is a notification")

        // 跑着的时候 describe 照样回；第二个 run 不收
        try host.send(request(.int(4), "describe"))
        try host.send(run(.int(5)))
        let first = try await host.lines.reply()
        let second = try await host.lines.reply()
        #expect(first["id"] == .int(4))
        #expect(second["id"] == .int(5))
        #expect(failureCode(second) == "invalid-args")

        // 别的 id 的 cancel 不算
        try host.send(cancel(.int(99)))
        try host.send(cancel(.int(3)))
        let reply = try await host.lines.reply()
        #expect(reply["id"] == .int(3))
        #expect(failureCode(reply) == "cancelled")
        #expect(reply["error"]?["message"] == .string("cancelled"))
        let end = try await host.close()
        #expect(throws: Never.self) { try end.get() }
    }

    @Test func aCancelledRunAnswersCancelledEvenWhenItFinishes() async throws {
        let ignoresTheFlag = ClosureTransform { p, ctx in
            await ctx.progress(done: 0, total: 1, unit: "image")
            while !ctx.isCancelled { try? await Task.sleep(for: .milliseconds(5)) }
            return echoResult(p)
        }
        let host = LoopHost(ignoresTheFlag)
        try host.send(run(.int(1)))
        #expect(try await host.lines.next()["method"] == .string("progress"))
        try host.send(cancel(.int(1)))
        #expect(failureCode(try await host.lines.reply()) == "cancelled")
        _ = try await host.close()
    }

    @Test func malformedAndOversizedFrames() async throws {
        let host = LoopHost(echo)
        func code(_ m: JSON) -> JSON? { m["error"]?["code"] }

        host.sendRaw(Array("not json\n".utf8))
        var m = try await host.lines.next()
        #expect(code(m) == .int(-32700) && m["id"] == .null)

        host.sendRaw([0xFF, 0xFE, 0x0A])
        #expect(code(try await host.lines.next()) == .int(-32700))

        try host.send(.array([request(.int(1), "describe")]))
        #expect(code(try await host.lines.next()) == .int(-32600))

        try host.send(.object(["jsonrpc": .string("1.0"), "id": .int(2), "method": .string("describe")]))
        m = try await host.lines.next()
        #expect(code(m) == .int(-32600) && m["id"] == .int(2))

        try host.send(.object(["jsonrpc": .string("2.0"), "id": .object(["x": .int(1)]), "method": .string("describe")]))
        m = try await host.lines.next()
        #expect(code(m) == .int(-32600) && m["id"] == .null)

        try host.send(.object(["jsonrpc": .string("2.0"), "id": .int(3)]))
        #expect(code(try await host.lines.next()) == .int(-32600))

        try host.send(.object(["jsonrpc": .string("2.0"), "id": .int(4), "method": .string("readback"), "params": .object([:])]))
        #expect(code(try await host.lines.next()) == .int(-32601))

        try host.send(.object(["jsonrpc": .string("2.0"), "id": .int(5), "method": .string("run"), "params": .object(["want": .array([])])]))
        #expect(code(try await host.lines.next()) == .int(-32602))

        var emptyWant = params()
        if case .object(var o) = emptyWant { o["want"] = .array([]); emptyWant = .object(o) }
        try host.send(runMessage(id: .int(6), params: emptyWant))
        #expect(code(try await host.lines.next()) == .int(-32602))

        var oldKind = params()
        if case .object(var o) = oldKind { o["want"] = .array([.string("derived/ocr/*.json")]); oldKind = .object(o) }
        try host.send(runMessage(id: .int(7), params: oldKind))
        #expect(code(try await host.lines.next()) == .int(-32602))

        try host.send(.object(["jsonrpc": .string("2.0"), "id": .int(8), "method": .string("cancel"), "params": .object(["id": .int(1)])]))
        #expect(code(try await host.lines.next()) == .int(-32600))

        // 不认识的通知、空行不回
        try host.send(.object(["jsonrpc": .string("2.0"), "method": .string("shutdown")]))
        host.sendRaw(Array("\n\r\n".utf8))

        host.sendRaw([UInt8](repeating: 0x20, count: maxFrame + 1) + [0x0A])
        m = try await host.lines.next()
        #expect(code(m) == .int(-32600) && m["id"] == .null)

        // 之后照常
        try host.send(request(.int(9), "describe"))
        #expect(try await host.lines.next()["id"] == .int(9))

        // 最后一行没有换行也算一条，随后 EOF
        host.sendRaw(Array(#"{"jsonrpc": "2.0", "id": 10, "method": "describe"}"#.utf8))
        try host.closeInput()
        #expect(try await host.lines.next()["id"] == .int(10))
        let end = try await host.close()
        #expect(throws: Never.self) { try end.get() }
    }

    @Test func eofEndsTheLoop() async throws {
        let idle = LoopHost(echo)
        let end = try await idle.close()
        #expect(throws: Never.self) { try end.get() }

        // 跑着的时候关了 stdin：取消它，等它回完
        let host = LoopHost(untilCancelled)
        try host.send(run(.int(1)))
        #expect(try await host.lines.next()["method"] == .string("progress"))
        try host.closeInput()
        #expect(failureCode(try await host.lines.reply()) == "cancelled")
        let stopped = try await host.close()
        #expect(throws: Never.self) { try stopped.get() }
    }

    @Test func aRunThatIgnoresCancelDoesNotHoldTheLoop() async throws {
        // 不查取消、6 秒以后才回的处理函数；等的时候不占线程池
        let stuck = ClosureTransform { p, ctx in
            await ctx.progress(done: 0, total: 1, unit: "image")
            await withCheckedContinuation { (later: CheckedContinuation<Void, Never>) in
                DispatchQueue.global().asyncAfter(deadline: .now() + 6) { later.resume() }
            }
            return echoResult(p)
        }
        let host = LoopHost(stuck)
        try host.send(run(.int(1)))
        #expect(try await host.lines.next()["method"] == .string("progress"))
        let begun = ContinuousClock.now
        let end = try await host.close()
        let waited = ContinuousClock.now - begun
        guard case .failure(let e) = end, case .runDidNotStop = e as? ServeError else {
            Issue.record("expected runDidNotStop, got \(end)")
            return
        }
        #expect(waited >= stopGrace && waited < stopGrace + .seconds(1))
        // 处理函数 6 秒后还会写一次回答：管道留到那之后再关，免得写进别的测试重用的文件描述符。
        try await Task.sleep(for: .seconds(3))
        withExtendedLifetime(host) {}
    }
}
