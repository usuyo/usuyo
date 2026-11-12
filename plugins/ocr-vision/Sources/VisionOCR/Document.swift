import CoreGraphics
import Foundation

/// `ocr.json`（feature 003 §5.3，schema `urn:usuyo:schema:ocr:1`）。
public struct OcrDocument: Sendable, Encodable, Equatable {
    public var images: [OcrImage]

    public init(images: [OcrImage]) {
        self.images = images
    }

    /// 有没有一行的分数到 `noTextBelow`；一行都没有就是 `no-text`（feature 003 §6）。
    public var hasText: Bool {
        images.contains { $0.lines.contains { $0.confidence >= noTextBelow } }
    }
}

/// 分数低于它的行不算认出了字；只用来判 `no-text`，这些行照样写进文件。
public let noTextBelow = 0.3

/// 一张图上认出的字。图文的图和封面带 `imageKey`，关键帧带 `frameSec`。
public struct OcrImage: Sendable, Encodable, Equatable {
    /// `run` 的 `inputs` 下标，也是 `run.json` 里 `inputs` 的下标。
    public var input: Int
    public var imageKey: String?
    public var frameSec: Double?
    /// 按方向摆正以后的像素宽高。
    public var width: Int
    public var height: Int
    /// 从上到下。
    public var lines: [OcrLine]

    public init(input: Int, imageKey: String?, frameSec: Double?, width: Int, height: Int, lines: [OcrLine]) {
        self.input = input
        self.imageKey = imageKey
        self.frameSec = frameSec
        self.width = width
        self.height = height
        self.lines = lines
    }
}

public struct OcrLine: Sendable, Encodable, Equatable {
    public var text: String
    /// `[x, y, w, h]`，0–1，原点在左上角。
    public var bbox: [Double]
    /// Vision 自己的分，很粗，不跨引擎比。
    public var confidence: Double

    public init(text: String, bbox: [Double], confidence: Double) {
        self.text = text
        self.bbox = bbox
        self.confidence = confidence
    }
}

/// Vision 的框（0–1，原点在左下角）换成 `[x, y, w, h]`（原点在左上角），夹到 0–1，留 4 位小数。
/// 先把四条边各自夹住、取整，再算宽高，所以 `x + w`、`y + h` 也不超过 1。
public func topLeftBox(_ r: CGRect) -> [Double] {
    let left = unit(Double(r.minX))
    let right = unit(Double(r.maxX))
    let top = unit(1 - Double(r.maxY))
    let bottom = unit(1 - Double(r.minY))
    return [left, top, round4(max(right - left, 0)), round4(max(bottom - top, 0))]
}

/// 分数留 3 位小数、夹到 0–1：Vision 给的是 `Float`，0.3 直接转出来是 0.30000001192092896。
public func confidence(_ c: Float) -> Double {
    let d = (Double(c) * 1000).rounded() / 1000
    return d.isNaN ? 0 : min(max(d, 0), 1)
}

/// 一行按框从上到下排；同一高度从左到右，再按原来的先后。
public func topToBottom(_ lines: [OcrLine]) -> [OcrLine] {
    lines.enumerated().sorted { a, b in
        let (ya, yb) = (a.element.bbox[1], b.element.bbox[1])
        if ya != yb { return ya < yb }
        let (xa, xb) = (a.element.bbox[0], b.element.bbox[0])
        if xa != xb { return xa < xb }
        return a.offset < b.offset
    }.map(\.element)
}

private func unit(_ v: Double) -> Double {
    v.isNaN ? 0 : round4(min(max(v, 0), 1))
}

private func round4(_ v: Double) -> Double {
    (v * 10_000).rounded() / 10_000
}
