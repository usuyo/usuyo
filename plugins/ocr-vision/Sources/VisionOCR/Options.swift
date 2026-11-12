import TransformLoop

/// `run` 的 `options`：读、补默认、回显。回显的那份原样交回来照样认（宿主重做时就是这样）。
public struct OcrOptions: Sendable, Equatable {
    public enum Level: String, Sendable {
        case accurate, fast
    }

    public var level: Level
    /// 按先后，前面的优先。
    public var languages: [String]
    public var languageCorrection: Bool

    /// 默认的语言：简体中文、英文在前，再是繁体和粤语（feature 003 §3.2）。
    public static let defaultLanguages = ["zh-Hans", "en-US", "zh-Hant", "yue-Hans", "yue-Hant"]

    /// `supported` 回这一档 Vision 认得的语言。不认识的键、类型不对、这一档不认的语言都回 `invalid-args`。
    public init(_ map: [String: JSON], supported: (Level) throws(CallFailure) -> [String]) throws(CallFailure) {
        let known: Set = ["level", "languages", "languageCorrection"]
        if let unknown = map.keys.sorted().first(where: { !known.contains($0) }) {
            throw CallFailure(.invalidArgs, "unknown option `\(unknown.prefix(64))`")
        }
        switch map["level"] {
        case nil: level = .accurate
        case .string(let s)?:
            guard let l = Level(rawValue: s) else {
                throw CallFailure(.invalidArgs, "`level` must be `accurate` or `fast`")
            }
            level = l
        default: throw CallFailure(.invalidArgs, "`level` must be a string")
        }
        let available = try supported(level)
        switch map["languages"] {
        case nil:
            languages = Self.defaultLanguages.filter(available.contains)
        case .array(let items)?:
            var list: [String] = []
            for item in items {
                guard case .string(let tag) = item else {
                    throw CallFailure(.invalidArgs, "`languages` must be a list of strings")
                }
                guard available.contains(tag) else {
                    throw CallFailure(.invalidArgs, "`\(tag.prefix(35))` is not a language Vision reads at the `\(level.rawValue)` level")
                }
                if !list.contains(tag) { list.append(tag) }
            }
            languages = list
        default: throw CallFailure(.invalidArgs, "`languages` must be a list of strings")
        }
        if languages.isEmpty {
            throw CallFailure(.invalidArgs, "`languages` must not be empty")
        }
        switch map["languageCorrection"] {
        case nil: languageCorrection = true
        case .bool(let b)?: languageCorrection = b
        default: throw CallFailure(.invalidArgs, "`languageCorrection` must be a boolean")
        }
    }

    /// 这次实际用的参数，进 `run` 的结果。
    public var echoed: [String: JSON] {
        [
            "level": .string(level.rawValue),
            "languages": .array(languages.map(JSON.string)),
            "languageCorrection": .bool(languageCorrection),
        ]
    }
}
