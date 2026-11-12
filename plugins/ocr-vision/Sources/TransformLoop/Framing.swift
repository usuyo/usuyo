import Foundation

/// 一条消息最多多少字节（不算换行），同 plugin-kit。`run` 的参数只有路径和设置，用不了这么多。
public let maxFrame = 1 << 20

/// 读进来的一帧。
enum Frame: Sendable {
    case line([UInt8])
    /// 超过上限的一行，已经读掉丢了。
    case tooLong
    case eof
}

/// 读写 stdio 出的错。
public struct StdioError: Error, CustomStringConvertible {
    public let what: String
    public let errno: Int32

    public var description: String {
        "\(what): \(String(cString: strerror(errno)))"
    }
}

/// 从一个文件描述符按行读；超过上限的部分不留在内存里。
struct FrameReader {
    let fd: Int32
    private var buffer: [UInt8] = []
    private var eof = false

    init(fd: Int32) {
        self.fd = fd
    }

    /// 读到换行或 EOF。最后一行没有换行也算一条。
    mutating func next() throws(StdioError) -> Frame {
        var line: [UInt8] = []
        var tooLong = false
        var readAny = false
        while true {
            if buffer.isEmpty {
                if eof { break }
                try fill()
                if buffer.isEmpty { break }
            }
            readAny = true
            let newline = buffer.firstIndex(of: UInt8(ascii: "\n"))
            let chunk = buffer[..<(newline ?? buffer.endIndex)]
            if !tooLong {
                if line.count + chunk.count > maxFrame {
                    tooLong = true
                    line = []
                } else {
                    line.append(contentsOf: chunk)
                }
            }
            if let newline {
                buffer.removeSubrange(...newline)
                if line.last == UInt8(ascii: "\r") { line.removeLast() }
                return tooLong ? .tooLong : .line(line)
            }
            buffer.removeAll(keepingCapacity: true)
        }
        if !readAny { return .eof }
        return tooLong ? .tooLong : .line(line)
    }

    /// 读一块进 `buffer`；读到 EOF 时记下。
    private mutating func fill() throws(StdioError) {
        var chunk = [UInt8](repeating: 0, count: 64 * 1024)
        while true {
            let n = chunk.withUnsafeMutableBytes { read(fd, $0.baseAddress, $0.count) }
            if n > 0 {
                buffer.append(contentsOf: chunk[..<n])
                return
            }
            if n == 0 {
                eof = true
                return
            }
            if errno != EINTR { throw StdioError(what: "read stdin", errno: errno) }
        }
    }
}

/// stdout：一次写一整行。只由 `Session` 这个 actor 调，所以几行不会交错。
struct Sink: Sendable {
    let fd: Int32

    func send(_ message: JSON) throws {
        var bytes = [UInt8](try message.line())
        bytes.append(UInt8(ascii: "\n"))
        var offset = 0
        while offset < bytes.count {
            let n = bytes.withUnsafeBytes { buf in
                write(fd, buf.baseAddress.map { $0 + offset }, buf.count - offset)
            }
            if n < 0 {
                if errno == EINTR { continue }
                throw StdioError(what: "write stdout", errno: errno)
            }
            offset += n
        }
    }
}
