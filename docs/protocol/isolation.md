<a id="7-权限与隔离"></a>
## 7. 权限与隔离

还没做：沙箱和宿主的本机代理都没有；插件的进程放在自己的进程组里，环境照 [§3.13](fetch.md#313-宿主和-plugin-kit-现在做到的11-11)。代理做出来时，plugin-kit 的客户端要能经宿主的代理出站（另用一个专门的变量，不读 `HTTP_PROXY`），地址检查跟着挪到宿主一侧。

沿用 [ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离)：macOS 用 Seatbelt，Linux 用 bubblewrap（不可用时 Landlock），Windows 先只强制 Job Object 与环境变量；超时杀进程组；`allowlist` 网络只经宿主代理出站；界面按平台、按声明项显示「强制 / 部分 / 仅告知」，不一律写「已沙箱」。各声明项在各平台的落实程度见 [05 §4.5](../research/05-note-apps-and-plugins.md#45-权限声明与各平台落实程度)。

补设备类：

| `devices` | macOS | Linux | Windows |
|---|---|---|---|
| `adb` | 部分：插件只连宿主管的 adb 服务端口，不自己起 adb 服务 | 部分：同左 | 仅告知 |
| `ios-device` | 待核实：能否用 Seatbelt 只对声明的插件放行 usbmux 套接字 | 部分：bind usbmux 套接字 | 仅告知 |
| `screen-capture` | 仅告知 | 仅告知 | 仅告知 |
| `accessibility` | 仅告知 | 仅告知 | 仅告知 |
| `audio-capture` | 仅告知 | 仅告知 | 仅告知 |
| `clipboard` | 待核实：能否用 Seatbelt 拦下未声明插件的剪贴板服务 | 仅告知 | 仅告知 |

- macOS 的屏幕录制、辅助功能、系统录音权限是授给 usuyo 本身的。子进程按「负责进程」用这些授权：第三方工具 agent-desktop 的权限引导代码和故障文档都说授权记在启动它的 App 上，也说系统设置有时会单独列出子程序、要一起打开（第三方旁证，没有查到 Apple 文档）。所以 usuyo 一旦拿到，未声明的插件也可能用得上；不做按插件分开授权（[§11](tradeoffs.md#11-设计取舍)），先按「仅告知」显示。
- **adb 服务由宿主管。** 宿主自己起一个 adb 服务，用私有端口；这个服务进程的 `HOME` 指向本机配置目录下的一个子目录，让 adb 的密钥（`$HOME/.android/adbkey`）固定下来。不这样的话，插件的私有 `${HOME}` 每次调用都会生成新密钥，手机每次都要重新点授权（推断）。adb 密钥算凭据，放本机配置目录、权限 0600，不放 `${CACHE}`。插件只拿到 `ADB_SERVER_SOCKET` 和 `ANDROID_SERIAL`（[§3.2](fetch.md#32-调用约定)），沙箱只放行这个端口；无线调试的配对和连接在宿主界面里做，插件拿不到手机的局域网地址。连着几台设备时，插件经同一个服务仍能点名别的设备，所以是「部分」。插件用 scrcpy 时，宿主给它 `ADB=<解析出的 adb>`，免得两个版本的 adb 互相重启服务端（推断，待核实）。
- 用到 `devices` 的插件，安装时在权限页单独列出这几项；第一次真正用到时，宿主先请用户在系统设置里授权 usuyo。
- 剪贴板：Apple 给通用剪贴板加了「程序读之前先问」（[NSPasteboard.AccessBehavior](https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum)，macOS 26 上还是开发者预览），以后默认开不开待核实；开了以后弹窗大概率记在 usuyo 上（推断，和上一条同理），宿主在第一次用到 `clipboard` 时先跟用户说清楚：跑的时候会动剪贴板，跑完写回原来的内容。
- 声明了 `accessibility`、`screen-capture`、`audio-capture`、`clipboard` 的插件要连系统服务（推断）。宿主在这个插件的沙箱里放行对应的系统服务，具体放行哪些规则待核实，实现时先在 Seatbelt profile 里跑一次 agent-desktop 的只读快照试出来；做不到的话，这几项在 macOS 上不进 Seatbelt，界面显示「仅告知」。要紧的是声明了的插件不被挡住，而不是拦下未声明的插件。
- 宿主不代用户安装根证书、不向别的进程注入；需要这些的插件让用户自己装、自己负责。

<a id="71-常驻进程的路径"></a>
### 7.1 常驻进程的路径

Seatbelt 的 profile、bubblewrap 的 bind 和 Landlock 的规则都在进程启动时定下，之后不能再放行新路径。常驻的转换、导出进程和 `serve` 模式的获取进程每次调用拿到的却是新的条目路径、`out` 目录和凭据文件。为此宿主给每个常驻进程建一个暂存目录：

```
<tmp>/usuyo-proc-<id>/
  in/<callId>/<下标>/<文件名>  输入：条目文件、导入文件的副本；进程只读
  out/<callId>/                这次调用的输出目录；进程可写
  cred/<callId>/               凭据、cookies.txt、密钥、设置文件；进程可读写
  home/                        ${HOME}
  bin/                         运行时的符号链接或转发脚本
  tmp/                         ${TMP}，也是进程的工作目录
```

`bin/` 的做法见 [§2.13](manifest.md#213-运行时解析)。

- 进程启动时，沙箱只放行这个暂存目录，加上已声明的 `${CACHE}`、`${MODELS}`，以及这个进程的站点集合里 `browser-profile` 站点的 `${PROFILE}`（[§3.12](fetch.md#312-常驻模式-serve)）；`readback: true` 的导出插件另放行目标文件夹 `${TARGET}`，只读，给 `readback` 用（规则启动时就定下，所以整个进程都看得到；`run` 照 [§4.4](transform-export.md#44-导出的-run-与-readback) 不碰它）。不放行整个 `items/` 或临时目录的上级，插件拿不到资料库根目录（[§1](README.md#1-概念)）。目标文件夹是资料库、在资料库里或包含资料库时，宿主不放行，导出设置里也选不了（推断）。
- 每次调用，宿主把输入放进 `in/<callId>/<下标>/`，文件名照原样：先在同一文件系统上建硬链接（`in/` 对进程只读，改不到原文件），跨文件系统时复制。插件在 `out/<callId>/` 旁边的 `in/<callId>/` 找到这次的输入。转换进程的 `<callId>` 就是 `run` 的 JSON-RPC `id`。没有沙箱时（[§7](#7-权限与隔离)），硬链接进 `in/` 挡不住插件改到原文件：原样被改过靠 `usuyo.fetch.json` 里的哈希查出来（同步前和「检查与修复」时，[SPEC §1.2.2](../SPEC.md#122-raw-和-derived)），不靠挡；有沙箱以后照这一条；所以宿主尽量把暂存目录建在和资料库同一个文件系统上。新建 `out/<callId>/`，把凭据写进 `cred/<callId>/`。`run`、`fetch` 参数和 `context` 里的路径都指向暂存目录。
- 调用结束后，宿主删掉 `in/<callId>/` 和 `cred/<callId>/`，`out/<callId>/` 照 [§3.6](fetch.md#36-fetch-与输出目录) 检查后移走。进程退出时整个暂存目录删除。
- 一个常驻进程能看到它自己正在进行的各次调用的暂存内容，看不到别的进程的。宿主同一时间只向一个进程发一个请求，所以实际只有一份。
- 一次一进程的调用不需要这一层：启动时就知道这次的路径，`${TMP}` 下的布局照此类推。
