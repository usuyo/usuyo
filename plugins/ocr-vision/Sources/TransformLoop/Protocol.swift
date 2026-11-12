// 转换协议的消息（协议 §4.1、§4.2），照 crates/usuyo-plugin 的 protocol::transform 和它的
// JSON Schema（crates/usuyo-plugin/schema/transform-*.schema.json）。只放这个插件用得到的。

/// JSON-RPC 请求的 `id`：整数或字符串，原样回。
public enum RpcId: Sendable, Hashable, Codable {
    case number(Int64)
    case text(String)

    public init(from decoder: any Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let n = try? c.decode(Int64.self) {
            self = .number(n)
        } else {
            self = .text(try c.decode(String.self))
        }
    }

    public func encode(to encoder: any Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .number(let n): try c.encode(n)
        case .text(let t): try c.encode(t)
        }
    }

    /// 从读进来的值认 `id`；不是整数或字符串时为 `nil`。
    init?(_ value: JSON) {
        switch value {
        case .int(let n): self = .number(n)
        case .string(let s): self = .text(s)
        default: return nil
        }
    }

    var json: JSON {
        switch self {
        case .number(let n): .int(n)
        case .text(let t): .string(t)
        }
    }
}

/// 转换插件产出的东西（协议 §2.9）。
public enum Produce: String, Sendable, Codable {
    case asr, keyframes, ocr, readable, vision, summary, embeddings, play, probe
    case audioPcm16k = "audio:pcm16k"
}

/// 转换插件吃的东西（协议 §2.9）。
public enum ConsumeKind: String, Sendable, Codable {
    case mediaVideo = "media:video"
    case mediaAudio = "media:audio"
    case image, html, text, subtitle
    case derivedAsr = "derived:asr"
    case derivedOcr = "derived:ocr"
    case derivedVision = "derived:vision"
    case derivedKeyframes = "derived:keyframes"
    case audioPcm16k = "audio:pcm16k"
}

public enum InputRole: String, Sendable, Codable {
    case video, audio
}

/// `run` 的参数。
public struct RunParams: Sendable, Decodable {
    public var item: RunItem
    public var want: [Produce]
    public var inputs: [RunInput]
    /// 这次调用的输出目录，绝对路径；插件只写这里。
    public var out: String
    public var options: [String: JSON]

    public init(item: RunItem, want: [Produce], inputs: [RunInput], out: String, options: [String: JSON]) {
        self.item = item
        self.want = want
        self.inputs = inputs
        self.out = out
        self.options = options
    }

    enum CodingKeys: String, CodingKey {
        case item, want, inputs, out, options, provider
    }

    public init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        item = try c.decode(RunItem.self, forKey: .item)
        want = try c.decode([Produce].self, forKey: .want)
        inputs = try c.decode([RunInput].self, forKey: .inputs)
        out = try c.decode(String.self, forKey: .out)
        options = try c.decodeIfPresent([String: JSON].self, forKey: .options) ?? [:]
        // 这个插件的清单 `provider: none`；有也只查它是 JSON，不用。
        _ = try c.decodeIfPresent(JSON.self, forKey: .provider)
    }
}

public struct RunItem: Sendable, Decodable {
    public var id: String
    public var kind: String
    public var title: String
    public var lang: String?

    public init(id: String, kind: String, title: String, lang: String? = nil) {
        self.id = id
        self.kind = kind
        self.title = title
        self.lang = lang
    }
}

/// 一个输入：放进进程暂存目录 `in/<callId>/` 后的绝对路径，只读（协议 §7.1）。
public struct RunInput: Sendable, Decodable {
    public var kind: ConsumeKind
    public var mime: String?
    public var path: String
    public var snapshot: String?
    public var imageKey: String?
    public var frameSec: Double?
    public var role: InputRole?

    public init(kind: ConsumeKind, path: String, imageKey: String? = nil, frameSec: Double? = nil) {
        self.kind = kind
        self.path = path
        self.imageKey = imageKey
        self.frameSec = frameSec
    }
}

/// `describe` 的结果。
public struct Describe: Sendable, Encodable {
    public var `protocol`: Int = 1
    public var version: String
    /// 实际可用的模型或语言。
    public var models: [String]

    public init(version: String, models: [String]) {
        self.version = version
        self.models = models
    }
}

/// `health` 的结果。
public struct Health: Sendable, Encodable {
    public var ready: Bool
    public var checks: [HealthCheck]

    public init(ready: Bool, checks: [HealthCheck]) {
        self.ready = ready
        self.checks = checks
    }
}

public struct HealthCheck: Sendable, Encodable {
    public var name: String
    public var ok: Bool
    public var message: String?

    public init(name: String, ok: Bool, message: String? = nil) {
        self.name = name
        self.ok = ok
        self.message = message
    }
}

/// `run` 的结果。宿主照它写 `run.json`。
public struct RunResult: Sendable, Encodable {
    public var outputs: [RunProduct]
    public var tool: UpstreamVersion
    public var options: [String: JSON]
    public var marks: [Mark]
    public var empty: Bool
    public var reason: EmptyReason?

    public init(
        outputs: [RunProduct], tool: UpstreamVersion, options: [String: JSON],
        marks: [Mark] = [], empty: Bool = false, reason: EmptyReason? = nil
    ) {
        self.outputs = outputs
        self.tool = tool
        self.options = options
        self.marks = marks
        self.empty = empty
        self.reason = reason
    }
}

public struct RunProduct: Sendable, Encodable {
    public var kind: Produce
    /// 相对 `out`。
    public var path: String

    public init(kind: Produce, path: String) {
        self.kind = kind
        self.path = path
    }
}

public struct UpstreamVersion: Sendable, Encodable, Equatable {
    public var name: String
    public var version: String

    public init(name: String, version: String) {
        self.name = name
        self.version = version
    }
}

public enum EmptyReason: String, Sendable, Encodable {
    case noText = "no-text"
}

/// 很明显的标记（feature 003 §6）；这个插件只出 `no-text`，不带秒数。
public struct Mark: Sendable, Encodable {
    public var code: MarkCode

    public init(code: MarkCode) {
        self.code = code
    }
}

public enum MarkCode: String, Sendable, Encodable {
    case noText = "no-text"
}

/// 通知 `progress` 的参数。只用来判断卡住。
struct Progress: Encodable {
    var id: RpcId
    var done: Double
    var total: Double
    var unit: String
}

/// 错误对象（协议 §3.3）。插件自己的失败放在 JSON-RPC `error` 的 `data` 里。
public struct CallFailure: Error, Sendable, Encodable, Equatable {
    public var code: ErrorCode
    /// 给日志看的原因。
    public var message: String

    public init(_ code: ErrorCode, _ message: String) {
        self.code = code
        self.message = message
    }

    public static let cancelled = CallFailure(.cancelled, "cancelled")
}

/// 这个插件会报的错误码（协议 §3.3 的子集）。
public enum ErrorCode: String, Sendable, Encodable {
    case unsupportedInput = "unsupported-input"
    case invalidArgs = "invalid-args"
    case cancelled
    case `internal`
}

/// JSON-RPC 的错误码。
enum RpcCode {
    static let parseError: Int64 = -32700
    static let invalidRequest: Int64 = -32600
    static let methodNotFound: Int64 = -32601
    static let invalidParams: Int64 = -32602
    static let pluginFailure: Int64 = -32000
}
