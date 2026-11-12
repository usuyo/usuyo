// 框的换算、分数、排序，ocr.json 过 contract 的 schema、和 contract 的样例逐字段对得上。
import CoreGraphics
import Foundation
import Testing

@testable import TransformLoop
@testable import VisionOCR

@Suite struct Boxes {
    @Test func bottomLeftBecomesTopLeft() {
        // Vision：x 0.1、y 0.7（下边）、宽 0.5、高 0.2 → 上边在 1 - 0.9 = 0.1
        #expect(topLeftBox(CGRect(x: 0.1, y: 0.7, width: 0.5, height: 0.2)) == [0.1, 0.1, 0.5, 0.2])
        // 贴着底边的一行，换过来 y 接近 1
        #expect(topLeftBox(CGRect(x: 0, y: 0, width: 1, height: 0.05)) == [0, 0.95, 1, 0.05])
    }

    @Test func clampsToTheImage() {
        // 伸出左边和上边
        #expect(topLeftBox(CGRect(x: -0.1, y: 0.9, width: 0.3, height: 0.2)) == [0, 0, 0.2, 0.1])
        // 伸出右边和下边
        #expect(topLeftBox(CGRect(x: 0.8, y: -0.1, width: 0.4, height: 0.2)) == [0.8, 0.9, 0.2, 0.1])
        // 整个在图外
        #expect(topLeftBox(CGRect(x: 1.2, y: 1.5, width: 0.1, height: 0.1)) == [1, 0, 0, 0])
        // NaN 不进文件
        let nan = topLeftBox(CGRect(x: .nan, y: .nan, width: 0.1, height: 0.1))
        #expect(nan.allSatisfy { !$0.isNaN && (0...1).contains($0) })
    }

    @Test func roundsToFourPlacesAndStaysInside() {
        let b = topLeftBox(CGRect(x: 0.07222222521604936, y: 0.754166666875, width: 0.5388888888888889, height: 0.054166666666666696))
        #expect(b == [0.0722, 0.1917, 0.5389, 0.0541])
        let edge = topLeftBox(CGRect(x: 0.99996, y: 0, width: 0.00004, height: 0.99996))
        #expect(edge[0] + edge[2] <= 1 && edge[1] + edge[3] <= 1)
    }

    @Test func confidenceIsRoundedAndTheThresholdIsInclusive() {
        // Float 的 0.3 直接转出来是 0.30000001192092896
        #expect(confidence(0.3) == 0.3)
        #expect(confidence(0.5) == 0.5)
        #expect(confidence(1.2) == 1)
        #expect(confidence(-0.1) == 0)
        #expect(confidence(.nan) == 0)
        let at = OcrDocument(images: [image(lines: [line(confidence: confidence(0.3))])])
        #expect(at.hasText)
        let below = OcrDocument(images: [image(lines: [line(confidence: 0.299)]), image(lines: [])])
        #expect(!below.hasText)
        #expect(!OcrDocument(images: [image(lines: [])]).hasText)
    }

    @Test func linesGoTopToBottomThenLeftToRight() {
        let lines = [
            line("c", bbox: [0.1, 0.8, 0.2, 0.05]),
            line("b", bbox: [0.5, 0.2, 0.2, 0.05]),
            line("a", bbox: [0.1, 0.2, 0.2, 0.05]),
            line("d", bbox: [0.1, 0.8, 0.2, 0.05]),
        ]
        #expect(topToBottom(lines).map(\.text) == ["a", "b", "c", "d"])
    }
}

@Suite struct Contract {
    /// 照 contract 的样例 testdata/ocr.images.json 拼出同样的文档，编出来的 JSON 和样例一样。
    @Test func encodingMatchesTheContractSample() throws {
        let sample = try JSON.parse(Data(contentsOf: repo.appending(path: "crates/usuyo-contract/testdata/ocr.images.json")))
        let doc = OcrDocument(images: [
            OcrImage(
                input: 0, imageKey: "3f2a9c0d1e4b5a67", frameSec: nil, width: 1080, height: 1440,
                lines: [OcrLine(text: "水温 92°C，粉水比1:15", bbox: [0.0736, 0.2542, 0.5504, 0.0542], confidence: 0.5)])
        ])
        #expect(try JSON(encoding: doc) == sample)
    }

    /// 一份带图、带关键帧、带空行列表的文档过 ocr.schema.json。
    @Test func encodingPassesTheSchema() throws {
        let schemaURL = repo.appending(path: "crates/usuyo-contract/schema/ocr.schema.json")
        let schema = try JSON.parse(Data(contentsOf: schemaURL))
        let doc = OcrDocument(images: [
            OcrImage(
                input: 0, imageKey: "3f2a9c0d1e4b5a67", frameSec: nil, width: 1080, height: 1440,
                lines: [
                    OcrLine(text: "Pour over in 3 stages", bbox: topLeftBox(CGRect(x: 0.07, y: 0.61, width: 0.59, height: 0.05)), confidence: confidence(0.5)),
                    OcrLine(text: "边上", bbox: topLeftBox(CGRect(x: 0.9, y: -0.02, width: 0.3, height: 0.1)), confidence: confidence(0.3)),
                ]),
            OcrImage(input: 1, imageKey: nil, frameSec: 12.04, width: 960, height: 540, lines: []),
        ])
        let value = try JSON(encoding: doc)
        let problems = validate(value, against: schema)
        #expect(problems.isEmpty, "\(problems)")

        // 反过来也要查得出错：缺 `lines`、框伸出 1、`imageKey` 和 `frameSec` 都有
        var broken = value
        if case .object(var top) = broken, case .array(var images)? = top["images"], case .object(var first) = images[0] {
            first["frameSec"] = .double(1)
            first["lines"] = nil
            images[0] = .object(first)
            images.append(.object([
                "input": .int(2), "frameSec": .double(0), "width": .int(1), "height": .int(1),
                "lines": .array([.object(["text": .string("x"), "bbox": .array([.double(0.5), .int(0), .double(0.6), .double(1.1)]), "confidence": .double(1)])]),
            ]))
            top["images"] = .array(images)
            broken = .object(top)
        }
        #expect(validate(broken, against: schema).count >= 3)

        // 机器上有 jsonschema 命令行时再用它校一次（不为它加依赖）。
        let cli = URL(filePath: "/opt/homebrew/bin/jsonschema")
        guard FileManager.default.isExecutableFile(atPath: cli.path(percentEncoded: false)) else {
            print("skip the jsonschema CLI check: no \(cli.path(percentEncoded: false))")
            return
        }
        let file = FileManager.default.temporaryDirectory.appending(path: "ocr-vision-\(UUID().uuidString).json")
        try value.line().write(to: file)
        defer { try? FileManager.default.removeItem(at: file) }
        let p = Process()
        p.executableURL = cli
        p.arguments = ["-i", file.path(percentEncoded: false), schemaURL.path(percentEncoded: false)]
        p.standardError = FileHandle.nullDevice
        try p.run()
        p.waitUntilExit()
        #expect(p.terminationStatus == 0)
    }
}

private func image(lines: [OcrLine]) -> OcrImage {
    OcrImage(input: 0, imageKey: "3f2a9c0d1e4b5a67", frameSec: nil, width: 10, height: 10, lines: lines)
}

private func line(_ text: String = "字", bbox: [Double] = [0, 0, 1, 1], confidence: Double = 0.5) -> OcrLine {
    OcrLine(text: text, bbox: bbox, confidence: confidence)
}

/// 照 schema 查一个值，只认 ocr.schema.json 用到的那几个关键字；schema 里出现别的关键字时报出来，
/// 免得 schema 加了规则这里悄悄不查。
private func validate(_ value: JSON, against schema: JSON, at path: String = "$") -> [String] {
    guard case .object(let s) = schema else { return ["\(path): schema is not an object"] }
    let known: Set = [
        "$schema", "$id", "title", "description", "type", "required", "properties", "items",
        "minimum", "maximum", "minItems", "maxItems", "pattern", "oneOf", "not",
    ]
    var problems = s.keys.filter { !known.contains($0) }.map { "\(path): unchecked keyword `\($0)`" }
    if let type = s["type"]?.string {
        let ok: Bool
        switch (type, value) {
        case ("object", .object), ("array", .array), ("string", .string), ("integer", .int),
            ("number", .int), ("number", .double), ("boolean", .bool):
            ok = true
        default: ok = false
        }
        if !ok { return problems + ["\(path): not \(type)"] }
    }
    let number: Double? =
        switch value {
        case .int(let i): Double(i)
        case .double(let d): d
        default: nil
        }
    if let n = number {
        if case .int(let m)? = s["minimum"], n < Double(m) { problems.append("\(path): below \(m)") }
        if case .int(let m)? = s["maximum"], n > Double(m) { problems.append("\(path): above \(m)") }
    }
    if case .string(let str) = value, let pattern = s["pattern"]?.string,
        str.range(of: pattern, options: .regularExpression) == nil
    {
        problems.append("\(path): does not match \(pattern)")
    }
    if case .object(let o) = value {
        if case .array(let required)? = s["required"] {
            for case .string(let key) in required where o[key] == nil { problems.append("\(path): no `\(key)`") }
        }
        if case .object(let props)? = s["properties"] {
            for (key, sub) in props { if let v = o[key] { problems += validate(v, against: sub, at: "\(path).\(key)") } }
        }
    }
    if case .array(let items) = value {
        if case .int(let m)? = s["minItems"], items.count < m { problems.append("\(path): fewer than \(m) items") }
        if case .int(let m)? = s["maxItems"], items.count > m { problems.append("\(path): more than \(m) items") }
        if let sub = s["items"] {
            for (i, v) in items.enumerated() { problems += validate(v, against: sub, at: "\(path)[\(i)]") }
        }
    }
    if case .array(let options)? = s["oneOf"] {
        let passing = options.filter { validate(value, against: $0, at: path).isEmpty }.count
        if passing != 1 { problems.append("\(path): matches \(passing) of `oneOf`, not exactly one") }
    }
    if let not = s["not"], validate(value, against: not, at: path).isEmpty {
        problems.append("\(path): matches `not`")
    }
    return problems
}
