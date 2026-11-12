// 参数：默认值、回显交回来照样认、不认识的键和语言回 invalid-args。不调 Vision，语言表是假的。
import Testing

@testable import TransformLoop
@testable import VisionOCR

@Suite struct Options {
    /// 和这台 Vision 报的一样：fast 一档没有中文。
    private func supported(_ level: OcrOptions.Level) -> [String] {
        switch level {
        case .accurate: ["en-US", "fr-FR", "zh-Hans", "zh-Hant", "yue-Hans", "yue-Hant", "ja-JP"]
        case .fast: ["en-US", "fr-FR"]
        }
    }

    @Test func defaultsAndEcho() throws {
        let o = try OcrOptions([:], supported: supported)
        #expect(o.level == .accurate)
        #expect(o.languages == ["zh-Hans", "en-US", "zh-Hant", "yue-Hans", "yue-Hant"])
        #expect(o.languageCorrection)
        #expect(o.echoed["languages"] == .array(o.languages.map(JSON.string)))
        // 回显的那份交回来，读出一样的参数
        #expect(try OcrOptions(o.echoed, supported: supported) == o)
    }

    @Test func fastKeepsOnlyWhatItReads() throws {
        let o = try OcrOptions(["level": .string("fast")], supported: supported)
        #expect(o.languages == ["en-US"])
        #expect(try OcrOptions(o.echoed, supported: supported) == o)
    }

    @Test func givenValuesAreKeptInOrder() throws {
        let o = try OcrOptions(
            ["languages": .array([.string("ja-JP"), .string("zh-Hans"), .string("ja-JP")]), "languageCorrection": .bool(false)],
            supported: supported)
        #expect(o.languages == ["ja-JP", "zh-Hans"])
        #expect(!o.languageCorrection)
    }

    @Test(arguments: [
        ["threads": JSON.int(2)],
        ["level": .string("best")],
        ["level": .int(1)],
        ["languages": .string("zh-Hans")],
        ["languages": .array([])],
        ["languages": .array([.int(1)])],
        ["languages": .array([.string("tlh")])],
        ["level": .string("fast"), "languages": .array([.string("zh-Hans")])],
        ["languageCorrection": .string("yes")],
    ])
    func refused(_ map: [String: JSON]) {
        #expect(throws: CallFailure.self) { try OcrOptions(map, supported: supported) }
        do {
            _ = try OcrOptions(map, supported: supported)
        } catch {
            #expect(error.code == .invalidArgs)
        }
    }
}
