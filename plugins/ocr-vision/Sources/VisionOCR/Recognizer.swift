import CoreGraphics
import Foundation
import ImageIO
import TransformLoop
import Vision

/// 用的 `VNRecognizeTextRequest` 版本，进 `tool`。
public let requestRevision = VNRecognizeTextRequestRevision3

/// 一张图最多多少像素；更大的不解码，免得一张图吃光内存。
let maxPixels = 200_000_000

/// 读出来、按方向摆正前的图。
struct LoadedImage {
    let image: CGImage
    let orientation: CGImagePropertyOrientation
    /// 摆正以后的宽高。
    let width: Int
    let height: Int
}

/// 读一张图；不是 ImageIO 能读的图、或太大时为 `nil`。
func loadImage(_ url: URL) -> LoadedImage? {
    guard let source = CGImageSourceCreateWithURL(url as CFURL, [kCGImageSourceShouldCache: false] as CFDictionary),
        CGImageSourceGetCount(source) > 0,
        let props = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
        let w = props[kCGImagePropertyPixelWidth] as? Int, let h = props[kCGImagePropertyPixelHeight] as? Int,
        w > 0, h > 0, w <= maxPixels / h,
        let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
    else { return nil }
    let raw = props[kCGImagePropertyOrientation] as? UInt32 ?? 1
    let orientation = CGImagePropertyOrientation(rawValue: raw) ?? .up
    // 5–8 是转了 90 度的，宽高对调。
    let turned = [.leftMirrored, .right, .rightMirrored, .left].contains(orientation)
    return LoadedImage(
        image: image, orientation: orientation,
        width: turned ? image.height : image.width, height: turned ? image.width : image.height)
}

/// 这一档 Vision 认得的语言。
func supportedLanguages(_ level: OcrOptions.Level) throws(CallFailure) -> [String] {
    do {
        return try request(level).supportedRecognitionLanguages()
    } catch {
        throw CallFailure(.internal, "Vision did not list its languages: \(error.localizedDescription)")
    }
}

/// 认一张图上的字，从上到下。
func recognize(_ loaded: LoadedImage, _ options: OcrOptions) throws -> [OcrLine] {
    let req = request(options.level)
    req.recognitionLanguages = options.languages
    req.usesLanguageCorrection = options.languageCorrection
    let handler = VNImageRequestHandler(cgImage: loaded.image, orientation: loaded.orientation)
    try handler.perform([req])
    let lines = (req.results ?? []).compactMap { observation -> OcrLine? in
        guard let best = observation.topCandidates(1).first else { return nil }
        return OcrLine(
            text: best.string, bbox: topLeftBox(observation.boundingBox),
            confidence: confidence(best.confidence))
    }
    return topToBottom(lines)
}

private func request(_ level: OcrOptions.Level) -> VNRecognizeTextRequest {
    let req = VNRecognizeTextRequest()
    req.revision = requestRevision
    req.recognitionLevel = level == .fast ? .fast : .accurate
    return req
}

/// `tool`：Vision、系统版本和 build、请求的版本，例如 `macOS 15.6.1 (24G90), VNRecognizeTextRequest revision 3`。
func visionTool() -> UpstreamVersion {
    let v = ProcessInfo.processInfo.operatingSystemVersion
    var version = "macOS \(v.majorVersion).\(v.minorVersion).\(v.patchVersion)"
    // operatingSystemVersionString 形如 `Version 15.6.1 (Build 24G90)`。
    let full = ProcessInfo.processInfo.operatingSystemVersionString
    if let start = full.range(of: "(Build "), let end = full[start.upperBound...].firstIndex(of: ")") {
        version += " (\(full[start.upperBound..<end]))"
    }
    return UpstreamVersion(name: "Vision", version: "\(version), VNRecognizeTextRequest revision \(requestRevision)")
}
