// swift-tools-version: 6.2
// 预装的图里认字插件 builtin/ocr-vision：系统 Vision 认字，stdio 上说转换协议（协议 §4）。
import PackageDescription

let package = Package(
    name: "ocr-vision",
    platforms: [.macOS(.v15)],
    products: [
        .executable(name: "usuyo-plugin-ocr-vision", targets: ["usuyo-plugin-ocr-vision"])
    ],
    targets: [
        // JSON-RPC 循环和协议消息，不碰 Vision。
        .target(name: "TransformLoop"),
        // 读图、调 Vision、换算框、写 ocr.json。
        .target(name: "VisionOCR", dependencies: ["TransformLoop"]),
        .executableTarget(
            name: "usuyo-plugin-ocr-vision",
            dependencies: ["TransformLoop", "VisionOCR"]
        ),
        .testTarget(
            name: "OcrVisionTests",
            dependencies: ["TransformLoop", "VisionOCR", "usuyo-plugin-ocr-vision"]
        ),
    ],
    swiftLanguageModes: [.v6]
)
