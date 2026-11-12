import Foundation

/// 任意一个 JSON 值。`run` 的 `options` 是自由的键值，消息本身也先读成它再按协议查。
public enum JSON: Sendable, Equatable, Codable {
    case null
    case bool(Bool)
    case int(Int64)
    case double(Double)
    case string(String)
    case array([JSON])
    case object([String: JSON])

    public init(from decoder: any Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() {
            self = .null
        } else if let b = try? c.decode(Bool.self) {
            self = .bool(b)
        } else if let i = try? c.decode(Int64.self) {
            self = .int(i)
        } else if let d = try? c.decode(Double.self) {
            self = .double(d)
        } else if let s = try? c.decode(String.self) {
            self = .string(s)
        } else if let a = try? c.decode([JSON].self) {
            self = .array(a)
        } else {
            self = .object(try c.decode([String: JSON].self))
        }
    }

    public func encode(to encoder: any Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .null: try c.encodeNil()
        case .bool(let b): try c.encode(b)
        case .int(let i): try c.encode(i)
        case .double(let d): try c.encode(d)
        case .string(let s): try c.encode(s)
        case .array(let a): try c.encode(a)
        case .object(let o): try c.encode(o)
        }
    }

    /// 解析一段字节；不是 JSON（含不是 UTF-8）时抛错。
    public static func parse(_ bytes: some Collection<UInt8>) throws -> JSON {
        try JSONDecoder().decode(JSON.self, from: Data(bytes))
    }

    /// 把一个可编码的值转成 JSON 值。
    public init(encoding value: some Encodable) throws {
        self = try JSON.parse(JSON.encoder.encode(value))
    }

    /// 编成一行：键排好序，不转义 `/`，没有换行。
    public func line() throws -> Data {
        try JSON.encoder.encode(self)
    }

    /// 按协议的类型读回来（例如 `run` 的参数）。
    public func decode<T: Decodable>(as type: T.Type) throws -> T {
        try JSONDecoder().decode(type, from: line())
    }

    public subscript(key: String) -> JSON? {
        if case .object(let o) = self { o[key] } else { nil }
    }

    public var string: String? {
        if case .string(let s) = self { s } else { nil }
    }

    private static var encoder: JSONEncoder {
        let e = JSONEncoder()
        e.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
        return e
    }
}
