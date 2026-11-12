// 进程入口：在 stdin、stdout 上跑转换协议的循环，直到宿主关掉 stdin。日志只写 stderr。
import Foundation
import TransformLoop
import VisionOCR

// 宿主关了 stdout 时让 write 回 EPIPE、循环收尾，不被信号直接杀掉。
signal(SIGPIPE, SIG_IGN)

let plugin = OcrVision()
do {
    try await serve(describe: plugin.describe(), transform: plugin, input: STDIN_FILENO, output: STDOUT_FILENO)
    exit(EXIT_SUCCESS)
} catch {
    FileHandle.standardError.write(Data("usuyo-plugin-ocr-vision: \(error)\n".utf8))
    exit(EXIT_FAILURE)
}
