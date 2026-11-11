// Draws card.png: four lines of mixed Chinese/English text on a 1080x1440 canvas.
// Usage: swiftc card.swift -o card && ./card <out.png>
// (Compiled, not `swift card.swift`: the interpreter crashes inside AppKit here.)
import AppKit

let out = CommandLine.arguments[1]
let width = 1080, height = 1440
let cg = CGContext(
    data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: 0,
    space: CGColorSpace(name: CGColorSpace.sRGB)!,
    bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue)!

NSGraphicsContext.current = NSGraphicsContext(cgContext: cg, flipped: false)
cg.setFillColor(red: 0.97, green: 0.95, blue: 0.91, alpha: 1)
cg.fill(CGRect(x: 0, y: 0, width: width, height: height))

let lines = [
    "手冲咖啡三段式注水",
    "Pour over in 3 stages",
    "水温 92°C，粉水比 1:15",
    "闷蒸 30 秒 Bloom",
]
let attrs: [NSAttributedString.Key: Any] = [
    .font: NSFont(name: "PingFang SC", size: 64) ?? NSFont.systemFont(ofSize: 64),
    .foregroundColor: NSColor(srgbRed: 0.12, green: 0.12, blue: 0.12, alpha: 1),
]
// The context's origin is bottom-left; the first line is drawn nearest the top.
for (i, line) in lines.enumerated() {
    let y = CGFloat(height) - 360 - CGFloat(i) * 200
    (line as NSString).draw(at: NSPoint(x: 80, y: y), withAttributes: attrs)
}
NSGraphicsContext.current = nil

let png = NSBitmapImageRep(cgImage: cg.makeImage()!).representation(using: .png, properties: [:])!
try! png.write(to: URL(fileURLWithPath: out))
