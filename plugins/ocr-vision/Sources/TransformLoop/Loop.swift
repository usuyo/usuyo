import Foundation
import Synchronization

// 转换插件那一侧的 JSON-RPC 循环（协议 §4.1、§4.2），在线上的表现照 plugin-kit 的 `transform` 模块：
//
// - 一行一条消息，最长 1 MiB；更长的回 `-32600`，丢掉这一行接着读。
// - 同一时间只跑一个 `run`，在自己的任务里；`describe`、`health`、`cancel` 跑着的时候照样收。
// - 收到 `cancel` 以后，处理函数返回什么都回 `cancelled`。
// - stdin 关了：跑着的 `run` 收到取消，最多等 4 秒，然后返回。

/// 一个转换插件。
public protocol Transform: Sendable {
    /// 处理一条。只写 `params.out`；长的活隔一阵查一次 `context.checkCancelled()`。
    func run(_ params: RunParams, _ context: RunContext) async throws(CallFailure) -> RunResult

    /// `health` 的回答。
    func health() -> Health
}

/// 一次 `run` 拿到的：取消和发 `progress` 的口子。
public struct RunContext: Sendable {
    let id: RpcId
    let session: Session

    /// 宿主发过 `cancel`，或 stdin 已经关了。
    public var isCancelled: Bool { Task.isCancelled }

    /// 取消了就抛 `cancelled`。
    public func checkCancelled() throws(CallFailure) {
        if Task.isCancelled { throw .cancelled }
    }

    /// 发一条 `progress`。写不出去时不打扰处理函数：错误留着，循环下一次读到消息时返回。
    public func progress(done: Double, total: Double, unit: String) async {
        await session.progress(Progress(id: id, done: done, total: total, unit: unit))
    }
}

/// 循环自己出的错。
public enum ServeError: Error, CustomStringConvertible {
    case stdio(any Error)
    /// stdin 关了以后跑着的 `run` 不肯停。
    case runDidNotStop(Duration)

    public var description: String {
        switch self {
        case .stdio(let e): "stdio: \(e)"
        case .runDidNotStop(let d): "stdin closed and the running `run` did not stop within \(d)"
        }
    }
}

/// stdin 关了以后等跑着的 `run` 收尾多久。宿主关 stdin 后等 5 秒才发 SIGTERM（协议 §4.1）。
public let stopGrace: Duration = .seconds(4)

/// 在给定的文件描述符上跑循环，直到输入关掉。插件入口给 0 和 1，测试给管道。
public func serve(describe: Describe, transform: some Transform, input: Int32, output: Int32) async throws(ServeError) {
    let session = Session(describe: describe, transform: transform, sink: Sink(fd: output))
    let frames = FrameFeed(fd: input)
    for await frame in frames.stream {
        defer { frames.handled() }
        do {
            try await session.handle(frame)
        } catch {
            throw .stdio(error)
        }
        if case .eof = frame {
            return try await stop(session)
        }
    }
    // 读线程出错时流提前结束。
    if let e = frames.failure() { throw .stdio(e) }
}

/// stdin 关了：取消跑着的，等它的回答写完。
private func stop(_ session: Session) async throws(ServeError) {
    await session.cancelRunning()
    let deadline = ContinuousClock.now + stopGrace
    while await session.isRunning {
        if ContinuousClock.now >= deadline { throw .runDidNotStop(stopGrace) }
        try? await Task.sleep(for: .milliseconds(10))
    }
    do {
        try await session.takeFailure()
    } catch {
        throw .stdio(error)
    }
}

/// 一个专门的线程按帧读 stdin，交给循环；循环处理完一帧才读下一帧，读的阻塞不占并发线程池。
private final class FrameFeed: Sendable {
    let stream: AsyncStream<Frame>
    private let turn = DispatchSemaphore(value: 0)
    private let readFailure = ErrorBox()

    init(fd: Int32) {
        let (stream, continuation) = AsyncStream.makeStream(of: Frame.self)
        self.stream = stream
        let turn = self.turn
        let readFailure = self.readFailure
        let thread = Thread {
            var reader = FrameReader(fd: fd)
            while true {
                let frame: Frame
                do throws(StdioError) {
                    frame = try reader.next()
                } catch {
                    readFailure.set(error)
                    continuation.finish()
                    return
                }
                continuation.yield(frame)
                if case .eof = frame {
                    continuation.finish()
                    return
                }
                turn.wait()
            }
        }
        thread.name = "usuyo-transform-stdin"
        thread.start()
    }

    func handled() {
        turn.signal()
    }

    func failure() -> StdioError? {
        readFailure.get()
    }
}

/// 读线程出的错，交回循环。
private final class ErrorBox: Sendable {
    private let lock = Mutex<StdioError?>(nil)

    func set(_ e: StdioError) {
        lock.withLock { $0 = e }
    }

    func get() -> StdioError? {
        lock.withLock { $0 }
    }
}

/// 循环的状态：回答写到哪、正在跑哪个 `run`。
actor Session {
    private let describe: Describe
    private let transform: any Transform
    private let sink: Sink
    private var running: Running?
    /// 处理任务写 `progress` 或回答失败时记下的第一个错误。
    private var failed: (any Error)?

    /// 正在跑的那个 `run`。处理任务在写回答的同一步把它清掉，宿主收到回答后马上发的下一个 `run` 不会撞上它。
    private struct Running {
        let id: RpcId
        let task: Task<Void, Never>
        var cancelled = false
    }

    init(describe: Describe, transform: any Transform, sink: Sink) {
        self.describe = describe
        self.transform = transform
        self.sink = sink
    }

    var isRunning: Bool { running != nil }

    func handle(_ frame: Frame) throws {
        try takeFailure()
        let line: [UInt8]
        switch frame {
        case .eof:
            return
        case .tooLong:
            try sink.send(error(nil, RpcCode.invalidRequest, "message longer than \(maxFrame) bytes"))
            return
        case .line(let l):
            line = l
        }
        // 空行不回，同 Rust 的 `is_ascii_whitespace`。
        if line.allSatisfy({ [0x20, 0x09, 0x0A, 0x0C, 0x0D].contains($0) }) { return }
        let message: Message
        switch Message.parse(line) {
        case .success(let m): message = m
        case .failure(let bad):
            try sink.send(error(bad.id, bad.code, bad.message))
            return
        }
        guard let id = message.id else {
            // 通知没有回答；只认 `cancel`。
            if message.method == "cancel", let raw = message.params["id"], let target = RpcId(raw),
                running?.id == target
            {
                running?.cancelled = true
                running?.task.cancel()
            }
            return
        }
        let reply: JSON
        switch message.method {
        case "describe":
            reply = try ok(id, describe)
        case "health":
            reply = try ok(id, transform.health())
        case "run":
            guard let refused = start(id, message.params) else { return }
            reply = refused
        case "cancel":
            reply = error(id, RpcCode.invalidRequest, "`cancel` is a notification")
        default:
            reply = error(id, RpcCode.methodNotFound, "no method `\(String(message.method.prefix(64)))`")
        }
        try sink.send(reply)
    }

    /// 起一个处理任务；起不了时回要写回去的错误。
    private func start(_ id: RpcId, _ raw: JSON) -> JSON? {
        let params: RunParams
        do {
            params = try raw.decode(as: RunParams.self)
        } catch {
            return self.error(id, RpcCode.invalidParams, "run params: \(error)")
        }
        if params.want.isEmpty {
            return error(id, RpcCode.invalidParams, "run params: `want` is empty")
        }
        if running != nil {
            return failure(id, CallFailure(.invalidArgs, "a `run` is already in progress; send one at a time"))
        }
        let context = RunContext(id: id, session: self)
        let transform = self.transform
        // 处理任务在这一步之后才能进 actor（`finish`），所以 `running` 一定先设好。
        let task = Task.detached {
            let reply: Result<RunResult, CallFailure>
            do throws(CallFailure) {
                reply = .success(try await transform.run(params, context))
            } catch {
                reply = .failure(error)
            }
            await context.session.finish(id, reply)
        }
        running = Running(id: id, task: task)
        return nil
    }

    private func finish(_ id: RpcId, _ outcome: Result<RunResult, CallFailure>) {
        let cancelled = running?.cancelled ?? false
        running = nil
        let reply: JSON
        if cancelled {
            reply = failure(id, .cancelled)
        } else {
            switch outcome {
            case .success(let result):
                do {
                    reply = try ok(id, result)
                } catch {
                    reply = failure(id, CallFailure(.internal, "encode the result: \(error)"))
                }
            case .failure(let f):
                reply = failure(id, f)
            }
        }
        remember { try sink.send(reply) }
    }

    func progress(_ p: Progress) {
        remember {
            try sink.send(.object(["jsonrpc": .string("2.0"), "method": .string("progress"), "params": JSON(encoding: p)]))
        }
    }

    func cancelRunning() {
        running?.cancelled = true
        running?.task.cancel()
    }

    func takeFailure() throws {
        if let e = failed {
            failed = nil
            throw e
        }
    }

    private func remember(_ write: () throws -> Void) {
        do {
            try write()
        } catch {
            if failed == nil { failed = error }
        }
    }

    private func ok(_ id: RpcId, _ result: some Encodable) throws -> JSON {
        .object(["jsonrpc": .string("2.0"), "id": id.json, "result": try JSON(encoding: result)])
    }

    private func failure(_ id: RpcId, _ f: CallFailure) -> JSON {
        let data = (try? JSON(encoding: f)) ?? .null
        return .object([
            "jsonrpc": .string("2.0"), "id": id.json,
            "error": .object([
                "code": .int(RpcCode.pluginFailure), "message": .string(f.code.rawValue), "data": data,
            ]),
        ])
    }

    private func error(_ id: RpcId?, _ code: Int64, _ message: String) -> JSON {
        .object([
            "jsonrpc": .string("2.0"), "id": id?.json ?? .null,
            "error": .object(["code": .int(code), "message": .string(message)]),
        ])
    }
}

/// 一条读进来的消息。
struct Message {
    var id: RpcId?
    var method: String
    var params: JSON

    struct Bad: Error {
        /// 能认出的 `id`；认不出回 `null`。
        var id: RpcId?
        var code: Int64
        var message: String
    }

    /// 解析一行，规则同 plugin-kit：先认 `id`，再查 `jsonrpc` 和 `method`。
    static func parse(_ line: [UInt8]) -> Result<Message, Bad> {
        let value: JSON
        do {
            value = try JSON.parse(line)
        } catch {
            return .failure(Bad(id: nil, code: RpcCode.parseError, message: "not JSON"))
        }
        guard case .object(let map) = value else {
            return .failure(Bad(id: nil, code: RpcCode.invalidRequest, message: "a message is one JSON object; batches are not supported"))
        }
        var id: RpcId?
        if let raw = map["id"] {
            guard let parsed = RpcId(raw) else {
                return .failure(Bad(id: nil, code: RpcCode.invalidRequest, message: "`id` must be an integer or a string"))
            }
            id = parsed
        }
        guard map["jsonrpc"]?.string == "2.0" else {
            return .failure(Bad(id: id, code: RpcCode.invalidRequest, message: "`jsonrpc` must be \"2.0\""))
        }
        guard let method = map["method"]?.string else {
            return .failure(Bad(id: id, code: RpcCode.invalidRequest, message: "no string `method`"))
        }
        return .success(Message(id: id, method: method, params: map["params"] ?? .null))
    }
}
