<a id="8-打包发布与分发"></a>
## 8. 打包、发布与分发

<a id="81-仓库命名"></a>
### 8.1 仓库命名

| 来源 | 仓库名 | 例子 |
|---|---|---|
| 官方组织 `usuyo` | 应用 `usuyo`、`plugin-kit`、转换与导出插件 `<kind>-<name>`，不加 `usuyo-` 前缀 | `usuyo/transform-asr-sherpa`、`usuyo/export-obsidian` |
| 获取方式组织 `usuyo-fetch` | 只放获取插件，`<platform>` 或 `<platform>-<取法>`，不加前缀 | `usuyo-fetch/douyin`、`usuyo-fetch/xiaohongshu-share`、`usuyo-fetch/takeout` |
| 个人仓库 | `usuyo-fetch-<name>`、`usuyo-transform-<name>`、`usuyo-export-<name>` | `someone/usuyo-fetch-douyin` |

所有插件仓库都加 GitHub topic `usuyo-plugin`。插件的 `id` 就是仓库的 `owner/repo`。

<a id="82-发版与-release-资产"></a>
### 8.2 发版与 release 资产

一个 release 里必须有：

| 资产 | 内容 |
|---|---|
| `usuyo-plugin.yaml` | 补好 `install` 的清单 |
| 插件本体，名字以清单里的 `install.assets[].name` 为准；建议写成 `<repo>_<version>_<os>-<arch>.tar.gz`（Windows 用 `.zip`）或 `<repo>_<version>_any.tar.gz` | 插件本体 |
| `provenance.sigstore.json` | 构建来源证明，覆盖上面所有资产；官方组织必须有，`usuyo-fetch` 自己约定都有，个人仓库建议有 |

- tag 写 `v<version>`；仓库开 GitHub immutable releases：发布后资产不能改、tag 不能移（2025-10-28 起正式可用，组织级可以统一打开）。所以要先建草稿、传齐资产、再发布。
- 用 plugin-kit 的可复用发版工作流：校验源清单、核对 tag 与 `version`、算 sha256 写进 `install`、生成来源证明、建草稿、上传、发布。插件仓库只写构建这一段：

```yaml
name: release
on:
  push:
    tags: ["v*"]
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<完整 commit SHA>
      - run: make dist
      - uses: actions/upload-artifact@<完整 commit SHA>
        with: {name: dist, path: dist/}
  release:
    needs: build
    permissions:
      contents: write
      id-token: write
      attestations: write
    uses: "usuyo/plugin-kit/.github/workflows/release.yml@<完整 commit SHA>"
```

所有 action 和可复用工作流都按完整 commit SHA 引用，由 Dependabot 升级。`usuyo-fetch` 是引用这个工作流，还是复制一份放进自己组织，见 [§8.8](#88-获取方式组织的仓库结构) 和 [§13](open.md#13-待定) 第 13 项。

不用这个工作流的（GoReleaser 或手工发版），用 `usuyo plugin pack --version 0.3.1 dist/*`：校验源清单、算 sha256，写出补好 `install` 的 `usuyo-plugin.yaml`，由作者自己连同资产上传。

<a id="83-校验与来源证明"></a>
### 8.3 校验与来源证明

客户端安装或更新一个版本，依次做：

0. **认准仓库。** 安装时记下 GitHub 的数字仓库 id 和所有者 id：来源是组织时从 `GET /orgs/{org}/repos` 的列表结果取，单独添加 `owner/repo` 时调一次 `GET /repos/{o}/{r}`。更新时，有来源证明的比对证书里的 Source Repository Identifier 和 Source Repository Owner Identifier；没有来源证明的，在检查更新时比对列表结果（单个仓库调一次 `GET /repos/{o}/{r}`）。下载清单时不跟随指向别的 `owner/repo` 的重定向。名字或数字 id 对不上时（仓库改名、转移，或旧名字被别人重新注册），标「来源已变：现在是 <新名字>」，停止更新，既不拒装也不自动装，让用户重新添加来源。新版本的 `version` 不高于已装版本时不更新，作者把旧版本标成 latest 也不会让客户端退回去。
1. 查屏蔽清单（[§8.6](#86-屏蔽清单与下架)），命中就拒装。
2. 下载 release 资产 `usuyo-plugin.yaml`，按 schema 校验；`id` 等于仓库、`version` 等于 tag，否则拒装。
3. 下载本机平台的资产，sha256 和清单不一致就拒装（[ADR-003 §6](../decisions/ADR-003-plugins.md#6-分发与校验)）。
4. 有 `provenance.sigstore.json` 时，用 Sigstore 的客户端库验证（用哪个 Rust 库实现时定），不经 GitHub API（Sigstore 的信任根由客户端按 TUF 定期更新）：签名有效、证书里的源仓库是这个仓库、源 ref 是这个 tag。签发者（构建它的工作流，按仓库和路径，不含 ref）的要求分两种：
   - **官方组织**：客户端内置，签发者必须是 `usuyo/plugin-kit` 的发版工作流；没有来源证明或验证失败就拒装。
   - **用户添加的来源**（包括 `usuyo-fetch`）：添加时固定签发者。[附录 A.4](#a4-添加第三方来源时的确认) 的确认框显示这个来源现有插件的签发工作流，例如「由 `usuyo/plugin-kit` 的 `release.yml` 构建」，或「没有来源证明」；用户添加后客户端记下。以后这个来源里的新版本，签发工作流和记下的不同，或者原来有来源证明现在没有，就不更新并标出来。添加时记下的是「没有来源证明」的，不做这项比对，只逐个版本显示有没有。验证失败一律拒装。这和 Mihon 的做法同一个思路：信任来自用户添加 store 时接受的签名指纹（[02 §15.7](../research/02-adapter-ecosystems.md#157-适配器源信任与治理)），客户端不用认识 `usuyo-fetch`。
5. 读 GitHub App 写在 tag 提交上的 check run，显示「校验通过 / 没通过 / 没有校验」（[ADR-003 §6](../decisions/ADR-003-plugins.md#6-分发与校验)）。只认 `app.id` 等于 usuyo GitHub App 的 check run，客户端内置这个 id：任何人都能用 Actions 的 `GITHUB_TOKEN` 或自己的 App 写一个同名、结论为 success 的 check run。官方组织的插件没通过就拒装；读不到 check run 时（App 还没跑完、被限额挡住）推迟这次更新，下次检查时再读，不安装。第三方插件读不到时显示「没有校验」；没通过时只标出，安装前多一步确认。
6. 展示权限，用户同意后安装（[§8.5](#85-添加来源更新与权限扩大)）。权限页列出每个站点的 `site`、`credential` 和 `domains`，以及每个密钥设置会被注入到哪个 `inject.host`。

分工：sha256 防下载途中被换；immutable releases 防发布后被换；来源证明说明资产是这个仓库在这个 tag 上由哪个工作流构建的，替代自己管签名密钥（Mihon 用 store 的签名指纹，[02 §15.7](../research/02-adapter-ecosystems.md#157-适配器源信任与治理)）；GitHub App 的 check run 说明清单和契约测试过没过。界面上把 [PRD §8.1](../PRD.md#711-设置) 里的「显示签名指纹」换成「由 `<owner>/<repo>` 的发版工作流构建」。

GitHub App 的检查加三条：`env` 里有凭据类变量判失败；`access` 或 `login` 比上一版高时写进检查结果，供更新说明显示；被包装工具里有发帖、点赞、私信一类命令时只作提示。App 不执行插件，具体分工见 [§10](plugin-kit.md#10-契约测试与-plugin-kit)。

第三方插件校验没通过时只标出，安装前多一步确认；屏蔽清单命中的照旧拒装。

<a id="84-三种来源"></a>
### 8.4 三种来源

| 来源 | 客户端里怎么来 | 标签 |
|---|---|---|
| 官方组织 `usuyo` | 客户端内置，列出组织里带 topic `usuyo-plugin` 的仓库；认 `kind: transform`、`export`，`kind: fetch` 只认下面说的两类预装插件，别的不列 | 官方 |
| 用户添加的组织，包括 `usuyo-fetch` | 用户在设置里输入组织名；列出组织里带 topic 的仓库 | 第三方来源 · 组织名 |
| 用户添加的单个仓库 | 用户输入 `owner/repo` | 第三方 · 未经 usuyo 审核 |

- 随安装包预装两类不含任何平台知识的获取插件（[PRD 7.2](../PRD.md#72-获取插件)）：通用网页获取（只做 `public-web`、不登录、没有站点专用规则，`network.hosts` 写 `"*"`，宿主代理照样挡住内网和云元数据地址），和读别的工具导出文件的导入（Netscape HTML 书签、Pocket、Omnivore、Readwise、Instapaper、Wallabag、Obsidian Web Clipper、flomo 一类，`network: none`）。它们的源码在官方组织，`id` 用 `builtin/` 前缀，跟着客户端一起发版和更新。平台官方导出包（抖音个人信息下载、Google Takeout、X 归档一类）的导入插件仍放 `usuyo-fetch`。
- 除了这两类，客户端不预置任何获取来源，`usuyo-fetch` 也要用户手动添加，代码里不对它特殊处理；对它的要求和别的用户来源一样，靠添加时固定的签发者（[§8.3](#83-校验与来源证明) 第 4 步）。
- 浏览第三方插件靠 GitHub 搜索 topic `usuyo-plugin`，客户端里给一个打开搜索结果的链接，不做商店。
- 每个平台的插件列表显示：名字、`access`、会用到哪个账号（`sites[].name`，没有就写「不用账号」）、来源标签、校验结果、来源证明、最近一次失败的原因。

插件列表里 `access` 显示一句白话，英文标识放进详情：official-export「读你导出的文件」，public-web「不登录，读公开页面」，vision-ui「看屏幕，替你点 App」，browser-session「在登录着的浏览器页面里读」，session-api「带你的登录信息直接请求平台」，private-api「模仿平台客户端发请求」，client-hook「读取或改动平台客户端本身」。

<a id="85-添加来源更新与权限扩大"></a>
### 8.5 添加来源、更新与权限扩大

- **添加第三方来源时确认一次声明**（文案见 [附录 A.4](#a4-添加第三方来源时的确认)），之后装这个来源里的插件不再逐个弹窗。每个来源只确认一次。
- **安装**：展示权限，用户按资料库同意（[ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离)）。
- **更新**：沿用「自动更新获取方式」的设置。新版本先过 [§8.3](#83-校验与来源证明) 的校验，再和已装版本比：
  - **扩大了权限**的，这个版本先停用，等用户重新同意，不静默更新。按集合和已装版本比，出现已装版本没有的值就算：`fs` 多了占位符；`network` 从 `none` 变 `allowlist`，或多了主机（包括 `"*"`）；`exec` 多了程序（包括 `requires.runtime` 新增的名字）；`env` 多了变量；`gpu` 从关到开；`devices` 多了项；`sites` 的 (`site`, `credential`, `domains`) 三元组里出现新值，例如多了站点、已有站点加了 `domains`、`credential` 从 `cookies` 改成 `browser-profile`；`settings` 里多了 `secret: true` 的键，或已有密钥的 `inject.host`、`inject.header` 改了。这几种改动都会把凭据或密钥交到新的地方：在「复用日常浏览器」下，多一项 `domains` 等于宿主替插件从日常浏览器多读一批 cookie；`browser-profile` 交出的是整个可写的资料目录。这是防经自动更新下发恶意代码的安全规则。
  - **调高了 `access` 或 `login`**、新增了 `unfavorite` 的，照常更新，只在更新说明里标出；`unfavorite` 仍默认关。
- **只从来源更新**：插件只从安装它的那个仓库更新（Mihon v0.21 的做法）。仓库改名、转移或被别人重新注册时怎么办，见 [§8.3](#83-校验与来源证明) 第 0 步。

自动更新默认开，不自动降级；用户可以在插件详情里手动回到上一版。声明带版本号：已添加的来源不重新确认；声明有实质变化时，只在下一次添加新来源时确认新版，并在应用的更新说明里写一句。

<a id="86-屏蔽清单与下架"></a>
### 8.6 屏蔽清单与下架

- 屏蔽清单 `security/plugin-blocklist.json` 放在主仓库，只列恶意的仓库和版本（[ADR-003 §6](../decisions/ADR-003-plugins.md#6-分发与校验)）。客户端在启动、安装、更新前读取并缓存；命中的拒装，已装的停用。
- 地址 `https://raw.githubusercontent.com/usuyo/usuyo/main/security/plugin-blocklist.json`，带 ETag 缓存，最多每 24 小时刷新一次。读不到时（离线、GitHub 不可达）用缓存；连缓存也没有时照常启动，安装前提示「屏蔽清单读不到」。
- `versions` 用 semver 范围语法（同 npm 的 node-semver），省略表示所有版本。条目带 `repoId`，按数字仓库 id 匹配，`repo` 只供人看（[§8.3](#83-校验与来源证明) 第 0 步）。
- 命中已装版本时，宿主结束它的常驻进程，取消它排队中的调用，再停用。

```json
{
  "updatedAt": "2026-11-01T00:00:00Z",
  "entries": [
    {"repo": "someone/usuyo-fetch-example", "repoId": 912345678, "versions": "<=0.2.3", "reason": "版本里带了上传 cookie 的代码", "at": "2026-10-31"}
  ]
}
```

- 来源里已经没有的插件（仓库被下架、移出组织、删除），客户端标「来源里已没有」，不再更新，不自动卸载；已经存下的快照不受影响（Mihon 的 orphaned）。
- 获取方式组织下架一个插件后，记进置顶 issue「已下架的获取方式」，写明原因和日期。因权利人通知下架的不再加回来；因维护原因下架的写明原因，以后可以重新提交（Keiyoushi 的置顶 issue 就是这样分的，[02 §1](../research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）。

<a id="87-github-api-限额与缓存"></a>
### 8.7 GitHub API 限额与缓存

不登录时 REST API 每个 IP 每小时 60 次；带 ETag 的条件请求回 304 时不计次，但只对带 `Authorization` 的请求成立。所以客户端尽量少用 API：

| 要读的 | 怎么读 | 计不计 API 次数 |
|---|---|---|
| 组织里的插件仓库 | `GET /orgs/{org}/repos?per_page=100`，结果里带 `topics`，客户端自己筛 | 计，每个组织每 100 个仓库一次 |
| 最新版本的清单 | `https://github.com/<owner>/<repo>/releases/latest/download/usuyo-plugin.yaml`，走网页下载的重定向 | 不计 API |
| 资产和来源证明 | 清单里的资产名拼 `releases/download/v<version>/<name>` | 不计 API |
| check run | `GET /repos/{o}/{r}/commits/{tag}/check-runs` | 计；不登录能不能读到待核实（ADR-003 待核实） |
| 单独添加的仓库的数字 id | `GET /repos/{o}/{r}`，安装时和检查更新时各一次（[§8.3](#83-校验与来源证明) 第 0 步） | 计 |

- 组织列表缓存 24 小时；清单按版本缓存，immutable release 的同一版本永不过期；检查更新最多每 6 小时一次，另有手动刷新。
- 可选：用户在设置里填一个只读公开仓库的 GitHub 令牌，限额升到每小时 5000 次。不要求登录 GitHub。

<a id="88-获取方式组织的仓库结构"></a>
### 8.8 获取方式组织的仓库结构

选**每个插件一个仓库**，不学 keiyoushi/extensions-source 的大仓库。

| 方面 | 每个插件一个仓库 | 一个大仓库 |
|---|---|---|
| 下架粒度 | 投诉或律师函只影响那一个仓库 | 一个插件被投诉，处理对象是整个仓库；GitHub 的 DMCA 流程给所有者改动的机会，改不了就禁用仓库（细节待核实） |
| 发现与发布 | 客户端按 topic 列仓库、读各自的最新 release，不要索引文件 | 要按插件打 tag，还要一份索引告诉客户端有哪些插件，这正是 ADR-003 放弃的东西 |
| 共享代码 | 平台中立的放进 plugin-kit；同一平台不同取法之间的少量重复可以接受 | 放 `lib/` 里直接共享，Keiyoushi 的 `lib-multisrc` |
| CI | 每个仓库一行引用 plugin-kit 的可复用工作流；组织级规则统一保护分支和 tag；升级靠 Dependabot 批量提 PR | 一处配置；但每次改动要做路径过滤，一个插件的构建坏了会挡住别的插件 |
| 维护权限 | 每个仓库一个 team，可以单独交给某个维护者 | 要靠 CODEOWNERS 细分 |

代价是仓库多了以后，统一的改动要批量提 PR；可复用工作流把这件事压到改一个 SHA。

改造上游项目时，新建仓库再推送它的历史（保留许可证和版权声明），不用 GitHub 的 fork 按钮：fork 会进上游的派生网络。派生超过 100 个、且通知方主张派生同样侵权时，GitHub 按整个派生网络处理，被屏蔽的仓库返回 451（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)）；派生不到 100 个时怎么处理，待核实。

**和 Keiyoushi 的不同。** Keiyoushi 用自己的 CI 和自己的签名密钥构建、发布，Mihon 不拥有、也不关联任何扩展仓库（[02 §1](../research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）。本文现在的写法里，`usuyo-fetch` 的发版引用官方组织 `usuyo/plugin-kit` 的可复用工作流，来源证明里的签发者因此是官方组织的工作流，官方组织进了获取插件的发版链；usuyo 的 GitHub App 装在 `usuyo-fetch` 上只做读和校验，和任何第三方作者装它一样，不参与发版。另一种做法是把工作流复制进 `usuyo-fetch/.github`，签发者变成 `usuyo-fetch` 自己的工作流，代价是 plugin-kit 改了工作流要再同步一次。选哪种见 [§13](open.md#13-待定) 第 13 项。两个组织的 owner 目前是同一个人，这一点两种做法都改变不了（[§8.9](#89-组织设置清单)）。

<a id="89-组织设置清单"></a>
### 8.9 组织设置清单

两个组织都要设，标「获取」的只在 `usuyo-fetch`：

- [ ] `.github` 仓库的 `profile/README.md`：组织主页。`usuyo-fetch` 用 [附录 A.2](#a2-获取方式组织主页)。
- [ ] `.github` 仓库放默认的社区文件：`SECURITY.md`（漏洞走 GitHub 私密漏洞报告）、`CONTRIBUTING.md`（清单写法、`maintainers` 要本人同意、不收写操作和推广内容）、issue 模板（插件坏了、请求新平台；获取：权利人通知）、PR 模板。
- [ ] 获取：置顶 issue「已下架的获取方式」。
- [ ] 成员必须开双重验证；基础权限只读；只有 owner 能建仓库、转入仓库、改可见性。
- [ ] 组织级规则：`main` 禁止强推和删除、合并要走 PR；`v*` tag 只有发版的人能建，不能删也不能移。
- [ ] 组织级打开 immutable releases。
- [ ] Actions：只允许 GitHub 官方的 action 和发版用的可复用工作流（`usuyo/plugin-kit` 的，或按 [§13](open.md#13-待定) 第 13 项复制进 `usuyo-fetch/.github` 的），按完整 SHA 引用；`GITHUB_TOKEN` 默认只读；只有发版工作流拿 `contents: write`、`id-token: write`、`attestations: write`。
- [ ] 谁能发版：每个仓库一个 team，名单和清单的 `maintainers` 一致；能推 `v*` tag 的就是这个 team 加 owner。维护者多于一人后，发版用的 environment 要求另一人批准。
- [ ] 装上 usuyo 的 GitHub App（公开的 App，只读和写 check run，不执行插件）。
- [ ] 打开 secret scanning、push protection 和 Dependabot。
- [ ] 联系方式：权利人通知写信到 `{contact}`；两个组织用不用同一个地址、用不用你域名下的地址，见 [§13](open.md#13-待定) 第 1 项。
- [ ] 两个组织的 owner 目前是同一个人，隔离只到组织和仓库这一层（[06 §17.4](../research/06-platform-access-and-account-risk.md#174-fork-与许可证)）。声明里凡是说谁在维护的句子，都按组织的实际成员如实写（[附录 A](#附录-a-声明模板)）。

`usuyo-fetch` 不用用户的域名：不放文档、不用 `$id`、不链个人主页，README 只链官方仓库里的协议文档；插件清单照样引用官方 schema。

<a id="附录-a-声明模板"></a>
## 附录 A. 声明模板

以下都是草稿，未经法律意见。`usuyo`、`usuyo-fetch`、`{contact}` 换成实际值；A.2、A.3 里的 `{contact}` 是获取方式组织用的地址，和 A.1 是不是同一个见 [§13](open.md#13-待定) 第 1 项。链接写成完整 URL，复制到别处也能打开。

<a id="a1-主仓库-readme"></a>
### A.1 主仓库 README

```markdown
**用途与责任**

usuyo 是在你自己电脑上运行的开源软件，用来做个人备份、离线阅读和注意力整理：把你在各平台存下的内容收回到本机的资料库里，变成能搜、能整理的文字。

- usuyo 本身不含任何平台的获取代码，也不托管、不转发任何平台内容。从平台取内容的是获取插件。它们是独立的程序，放在单独的仓库和组织里，不随 usuyo 分发，要你自己添加。随 usuyo 一起装好的只有两类不含任何平台知识的插件：通用网页获取，和读别的工具导出文件（书签、Pocket、Readwise 一类）的导入。
- 用不用某个获取插件、用哪个账号，由你决定。平台可能因此限制或封禁你的账号；这些后果和相关的法律责任由你自己承担。各种获取方式的已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 请遵守各平台的用户协议，尊重内容作者和版权人。收回来的内容只供你自己查看，不要再发布或分发。
- usuyo 不提供默认开启的平台写操作。插件声明的「在平台上取消收藏」默认关闭，每次执行前要你确认。
- 权利人如认为本仓库或官方组织里的内容侵犯了权利，请写信到 {contact}，写明仓库、文件和理由。我们核实后处理，并回复处理结果。

本段是草稿，未经法律意见。
```

<a id="a2-获取方式组织主页"></a>
### A.2 获取方式组织主页

```markdown
**usuyo-fetch：usuyo 的获取插件**

这里每个仓库是一个平台上的一种取法，用来把你自己存下的内容做个人备份、离线阅读和注意力整理。

- 本组织和 usuyo 官方组织 usuyo 是两个独立的组织，仓库、发版和下架各自处理。usuyo 客户端不预置这里的任何插件；要用的话，在 usuyo 的设置里手动添加组织名 `usuyo-fetch`。
- 每个插件的清单写明获取层级（access）、要不要登录（login），以及会用到哪个站点的账号。挑哪个、用哪个账号由你决定；平台对账号的限制和相关的法律后果由你自己承担。已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 插件只读取你自己有权查看的内容。没有默认开启的写操作；包装别的工具时去掉了发帖、点赞、私信一类命令。
- 本组织不托管任何平台内容，release 里只有插件程序和清单。
- 请遵守各平台的用户协议，尊重内容作者和版权人。
- 权利人通知：写信到 {contact}，或用 issue 模板「权利人通知」（不想公开时用邮件），写明仓库、文件和理由。核实后我们下架对应的仓库，记进置顶 issue「已下架的获取方式」；因权利人通知下架的不再加回来。
- 安全问题请按 SECURITY.md 私下报告。

本段是草稿，未经法律意见。
```

<a id="a3-插件-readme-模板"></a>
### A.3 插件 README 模板

```markdown
**{插件名}**

- 平台：{平台}
- 获取层级：`{access}`；登录：`{login}`；会用到：{站点名}账号（`{site}`）。没有登录时写「不用账号」。
- 上游：{上游项目}（{许可证}）。本仓库的改动见 CHANGES.md。
- 用途：个人备份、离线阅读、注意力整理。只取你自己有权查看的内容，不发帖、不点赞、不私信。
- 账号与责任：平台可能限制或封禁账号。用不用、用哪个账号由你决定，后果由你自己承担。已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 平台写操作：{没有 / 「在平台上取消收藏」，默认关闭，每次执行前确认}。
- 不托管任何平台内容。请遵守平台协议，尊重作者和版权人。权利人通知：{contact}。

本段是草稿，未经法律意见。
```

<a id="a4-添加第三方来源时的确认"></a>
### A.4 添加第三方来源时的确认

客户端在用户添加组织名或 `owner/repo` 时显示，每个来源一次：

```text
标题：添加第三方来源 {来源}

{来源} 不由 usuyo 审核。里面的插件可能出错，也可能含有恶意代码；usuyo 会显示每个版本的校验结果、来源证明和要用的权限。

这个来源现有的插件：{由 <owner>/<repo> 的 <工作流> 构建 / 没有来源证明}。以后的新版本换了构建方式，usuyo 会停止更新并告诉你。

获取插件会用你的网络，有的还会用你的平台账号去取内容。请只用来做个人备份、离线阅读和注意力整理。平台对账号的限制和相关的法律后果，由你自己承担。（链接：各种获取方式的已知情况）

请遵守平台的用户协议，尊重作者和版权人，不要再发布收回来的内容。

[ ] 我读过了，使用这个来源的后果由我自己承担

按钮：添加 · 取消
```

「添加」在勾选后才能点。「各种获取方式的已知情况」链到 [06 §16](../research/06-platform-access-and-account-risk.md#16-账号风险的证据)。本段是草稿，未经法律意见。
