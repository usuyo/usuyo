<a id="10-契约测试与-plugin-kit"></a>
## 10. 契约测试与 plugin-kit

插件仓库里放：

| 文件 | 内容 | 什么时候跑 |
|---|---|---|
| `testdata/canonicalize.json` | `[{"input": "<链接>", "expect": {"match": true, "identity": {…}}}, {"input": "https://example.com/", "expect": {"match": false}}]`；要展开短链的标 `"live": true` | 离线部分每次 |
| `testdata/outputs/*.json` | 录下的 `fetch`、`list`、`sources`、`health` 输出；提交前换成样例数据，`usuyo plugin try --record` 默认把标题、作者和各种 ID 换成占位值，因为这些来自作者自己的账号，放进公开仓库就公开了 | 每次，只校验 schema 与路径 |
| `testdata/live.json` | 真实链接，给维护者在自己电脑上跑 | 只在本机加 `--live` 时 |

```
usuyo plugin check .
usuyo plugin check . --live
usuyo plugin try fetch "https://v.douyin.com/iRNBho6u/"
usuyo plugin pack --version 0.3.1 dist/*
```

`usuyo plugin check` 做：清单 schema；`id` 和 git 远端一致；占位符和 `${setting.*}` 都能解析；`access`、`login` 顶层是不是各能力里的最高档；`login: none` 的能力没有 `sites`；`fetch.input` 含 `url` 时有 `match`；`sites[].domains` 每项和 `site` 同属一个可注册域名、不是公共后缀；`env` 里没有凭据类变量；`outputs`、`produces` 用的是词表里的值；样例输出过 schema、路径不越界；在禁网的沙箱里跑离线的 `canonicalize` 样例，连同 plugin-kit 为每个声明的平台附的共用样例，`identity.contentId` 要符合平台列表里的格式；跑 `health`；`run.serve: true` 时两种方式结果一致；`readback: true` 的导出插件跑往返测试，`readback(run(V))` 按值等于 V 的用户层，字符串逐字节相同，集合相等（[§4.4](transform-export.md#44-导出的-run-与-readback)）。`usuyo plugin try` 在真实的沙箱和凭据下跑一次，打印信封、事件和被代理拦下的主机。`usuyo plugin pack` 给不用 plugin-kit 发版工作流的仓库生成发布清单（[§8.2](distribution.md#82-发版与-release-资产)）。

- 契约测试不在 CI 里请求平台：CI 在机房 IP 上跑，结果不稳，也平白多出请求。
- GitHub App 不执行插件，只做不用跑插件的检查：清单 schema、tag 和 `version`、资产 sha256、来源证明、`testdata/outputs` 过 schema，结果写成 check run。要执行插件的部分（`health`、禁网沙箱里的 `canonicalize` 样例、两种方式结果一致、导出插件的往返测试）放进 plugin-kit 的可复用工作流，在插件仓库自己的 Actions 里跑；App 只读那次运行的结论。否则 App 要在自己的服务器上执行任何装了它的仓库发布的二进制。
- plugin-kit（MIT）里有：清单、信封、JSON-RPC 消息和 `derived/` 文件的 JSON Schema；平台 id 列表；`usuyo plugin check` 的实现（主程序内嵌同一份代码，另有给 CI 用的 action）；发版可复用工作流；SDK。plugin-kit 现在是 Rust crate `crates/usuyo-plugin-kit`，转出协议类型，另带 HTTP 客户端（[§10.1](#101-plugin-kit-的-http-客户端11-11)）；TypeScript、Python 的 SDK 还没有。SDK 管参数解析、信封和错误码、stderr 事件、常驻模式的循环、凭据文件读取、输出目录的路径检查、转换与导出的 JSON-RPC 循环；作者只写处理函数。
- schema 以 plugin-kit 为准；内核经 cargo 依赖引用，`crates/usuyo-contract` 只留资料库自己的类型（[§11](tradeoffs.md#11-设计取舍)）。schema 文件现在在 `crates/usuyo-plugin/schema/`，还没有 `usuyo plugin check`、`try`、`pack`。

<a id="101-plugin-kit-的-http-客户端11-11"></a>
### 10.1 plugin-kit 的 HTTP 客户端（11-11）

`crates/usuyo-plugin-kit` 的 `http::Client`（MIT）。获取插件的每个请求都走它，包括接口调用、展开短链、下图片和视频，不另引别的 HTTP 库：在沙箱和宿主的代理做出来以前（[§7](isolation.md#7-权限与隔离)），这是挡住插件连到本机和内网的那一层。它做的：

- **只连公网地址。** 只认 `http`、`https`；带用户名、密码的链接不连；主机要在调用方给的名单里。解析出的每个地址都要是公网：IPv4 拒 `0/8`、`10/8`、`100.64/10`、`127/8`、`169.254/16`、`172.16/12`、`192.0.0/24`、`192.0.2/24`、`192.88.99/24`、`192.168/16`、`198.18/15`、`198.51.100/24`、`203.0.113/24`、`224/4`、`240/4`；IPv6 只认 `2000::/3`，再去掉 `2001::/23`、`2001:db8::/32`、`2002::/16`、`3fff::/20`；IPv4 映射和 `64:ff9b::/96` 按里面的 IPv4 判。连接只连查过的那个地址，不再查一次 DNS。DNS 算在总时限里。
- **假 IP 代理。** 宿主设了 `USUYO_NETWORK=fake-ip`（[§3.13](fetch.md#313-宿主和-plugin-kit-现在做到的11-11)）时，回答全在 `198.18.0.0/15` 的域名照常连；单段名字和 `.localhost`、`.local`、`.internal`、`.lan`、`.home`、`.home.arpa`、`.corp`、`.intranet`、`.private` 结尾的照旧拒；直接写成这一段 IP 的链接一律拒。插件用 `Policy::from_host()`（`Client::new` 就是），测试才自己拼策略。
- **重定向默认不跟。** 调用方开了才跟（只跟 301、302、303、307、308），每一跳重新查主机名单和地址。换了主机，或从 https 跳到 http，只留 `Accept`、`Accept-Language`，调用方加的别的头不带过去。
- **cookie** 不能自己写进 `Cookie` 头，要用 `Cookie::new(值, 主机)`：只发给列出的主机，只走 https。没有 cookie 罐，`Set-Cookie` 不带到下一跳。
- **大小和时间。** 读正文时调用方必须给上限，按解压后的字节算，到上限就停（`TooLarge`）；默认总时长 30 秒（含所有跳和读正文），连接 15 秒。不读系统的代理设置。

通用网页插件（`plugins/web`）另外：图片的 `Referer` 只给页面的源加 `/`，https 页面上的 http 图片不给；正文里有图片没下下来时在 `missingReason` 里写几张没下下来，一张都没下下来才把 `images` 记进 `missing`。设置 `allow-private-network`（默认关）只放开你给的那条链接的第一跳，图片和重定向照旧只连公网。
