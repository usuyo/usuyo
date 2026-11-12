import Foundation
import TransformLoop

/// 这次调用能碰的两个目录（协议 §7.1）：宿主在 `run` 里给 `out`，即 `<暂存>/out/<callId>/`；
/// 输入在同一个暂存目录的 `in/<callId>/` 下。别的路径一律不读不写。
public struct CallDirs: Sendable {
    /// 解开符号链接以后的 `out`。
    public let out: URL
    /// 解开符号链接以后的 `in/<callId>/`。
    public let inputs: URL

    public init(out: String) throws(CallFailure) {
        guard let outURL = plainAbsolute(out) else {
            throw CallFailure(.invalidArgs, "`out` must be an absolute path without `.` or `..`")
        }
        guard isDirectory(outURL) else {
            throw CallFailure(.invalidArgs, "`out` is not a directory")
        }
        let parent = outURL.deletingLastPathComponent()
        guard parent.lastPathComponent == "out" else {
            throw CallFailure(.invalidArgs, "`out` is not `<staging>/out/<callId>`")
        }
        let inURL = parent.deletingLastPathComponent()
            .appending(path: "in", directoryHint: .isDirectory)
            .appending(path: outURL.lastPathComponent, directoryHint: .isDirectory)
        guard isDirectory(inURL) else {
            throw CallFailure(.invalidArgs, "no `in/<callId>` next to `out`")
        }
        self.out = outURL.resolvingSymlinksInPath()
        self.inputs = inURL.resolvingSymlinksInPath()
    }

    /// 第 `index` 个输入的路径：要是绝对路径、普通文件、解开符号链接以后还在 `in/<callId>/` 里。
    /// 报错只写下标，不写路径。
    public func input(_ path: String, index: Int) throws(CallFailure) -> URL {
        guard let url = plainAbsolute(path) else {
            throw CallFailure(.invalidArgs, "input \(index) must be an absolute path without `.` or `..`")
        }
        let type = (try? FileManager.default.attributesOfItem(atPath: url.path(percentEncoded: false)))?[.type] as? FileAttributeType
        guard type == .typeRegular else {
            throw CallFailure(.invalidArgs, "input \(index) is not a regular file")
        }
        let resolved = url.resolvingSymlinksInPath()
        guard resolved.pathComponents.starts(with: inputs.pathComponents),
            resolved.pathComponents.count > inputs.pathComponents.count
        else {
            throw CallFailure(.invalidArgs, "input \(index) is outside this call's input directory")
        }
        return resolved
    }
}

/// 绝对路径、没有 `.`、`..` 段时回它的 URL。
private func plainAbsolute(_ path: String) -> URL? {
    guard path.hasPrefix("/"), !path.contains("\0") else { return nil }
    let parts = path.split(separator: "/", omittingEmptySubsequences: true)
    guard !parts.contains(where: { $0 == "." || $0 == ".." }) else { return nil }
    return URL(filePath: path)
}

/// 是目录本身，不是指向目录的符号链接。
private func isDirectory(_ url: URL) -> Bool {
    let type = (try? FileManager.default.attributesOfItem(atPath: url.path(percentEncoded: false)))?[.type] as? FileAttributeType
    return type == .typeDirectory
}
