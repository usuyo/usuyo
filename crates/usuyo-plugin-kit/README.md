# usuyo-plugin-kit

写 usuyo 获取插件时依赖它：

- `protocol`：插件协议 v1 的类型，原样转自 `usuyo-plugin`。
- `http`：发请求用的客户端。

```toml
[dependencies]
usuyo-plugin-kit = { path = "../xiecun/crates/usuyo-plugin-kit" }
```

## 每个请求都走 `http::Client`

用户交来的链接、平台回答里的图片和视频地址、服务器给的跳转去处，都是别人写的。插件照着去连，就可能被引到你电脑上的本机服务、家里的路由器和 NAS、公司内网，或者云主机的元数据接口。所以插件里的每个请求都用这个客户端发，包括接口调用、展开短链、下载图片和视频。不要另外引 `ureq`、`reqwest` 自己发。

## 它拒绝什么

| 情况 | 错误 | 为什么 |
|---|---|---|
| 不是 http(s)，或地址里带账号密码 | `Invalid` | 只取网页和文件，不让地址里夹带凭据 |
| 主机不在你给的名单里 | `NotAllowed` | 插件只该连它声明过的那几台主机 |
| 域名解析出的地址里有一个不是公网 | `NotPublic` | 挡住回环、内网、链路本地（含 `169.254.169.254`）、运营商 NAT、组播和保留段；IPv4 映射和 NAT64 写法按里面的 IPv4 判断 |
| 跳转次数超过上限 | `TooManyRedirects` | 防止无限跳转 |
| 自己写了 `Cookie` 请求头 | `CookieHeader` | cookie 要限定发给哪些主机，只能经 `Request::cookie` 带 |
| 响应体超过上限 | `TooLarge` | 防止一个回答把磁盘或内存撑满 |

另外几条：

- 检查过的地址就是要连的地址，不再查第二次 DNS，域名解析中途换成内网地址也没用。
- 开着「假 IP」模式的代理（Clash、Surge、sing-box 一类）时，系统给每个域名都回 `198.18.0.0/15` 里的地址。是不是这种环境由宿主拿固定的公网域名去查，查到了经 `USUYO_NETWORK=fake-ip` 告诉插件；这时域名查出来全在这一段的照常放行，否则在这样的机器上什么都取不了。没有这个变量时这一段照旧拒绝。代价：这时看不到真正的地址，一个真指向内网的域名拦不住，要靠代理工具自己的规则；客户端按名字先挡掉单段名和 `.local`、`.lan`、`.internal` 这类一看就是内网的，链接里直接写这一段的数字地址照样拒绝。
- 跳转默认不跟，3xx 原样交回给你，由你看 `Location` 决定下一步。调用 `follow_redirects(n)` 才跟，每一跳都重新过上面的检查。
- cookie 只在 https 上发，同一个主机跳到明文 http 也不带。
- 不读 `HTTP_PROXY` 一类环境变量，没有 cookie 罐：上一跳的 `Set-Cookie` 不会带到下一跳。
- `timeout` 管一次请求的总时长，包括所有跳转和读完响应体，默认 30 秒。
- 响应体只能带上限读：`read_to_vec(limit)`，或大文件用 `into_reader(limit)`，超过上限时读错误的类别是 `io::ErrorKind::FileTooLarge`。
- `Error::is_refusal()` 为真的，是检查没过、根本没有连出去。

唯一的例外：`allow_private_first_hop(true)` 让请求的地址本身可以是本机或内网，只放开这一跳的地址段检查，跳转目标照样只连公网。只在用户自己在设置里打开时用，例如通用网页插件的「允许取内网网页」。

## 声明能连的主机

一个客户端带一份名单。名单越窄越好：按请求的用途分开建客户端，接口一个、短链一个、图片视频一个。

```rust
use usuyo_plugin_kit::http::{Client, Cookie, Hosts};

const UA: &str = "Mozilla/5.0 (…) Chrome/142.0.0.0 Safari/537.36";

let api = Client::new(UA, Hosts::only(["api.bilibili.com"]));
let media = Client::new(UA, Hosts::only([".hdslb.com", ".bilivideo.com"]));

let mut reply = api
    .get(&url)
    .header("Referer", "https://www.bilibili.com/")
    .cookie(&Cookie::new(cookie_header, ["api.bilibili.com"]))
    .timeout(std::time::Duration::from_secs(20))
    .send()?;
let body = reply.read_to_vec(8 << 20)?;
```

- `api.bilibili.com` 只匹配这一个主机名。
- `.hdslb.com` 只匹配它的子域，例如 `i0.hdslb.com`，不含 `hdslb.com` 本身；两个都要就都写上。
- 只比主机名，不比端口。
- 名单要和清单里 `permissions.network.hosts` 对得上：清单写 `*.hdslb.com`，这里写 `.hdslb.com`。
- `Hosts::any()` 不限主机，只看地址是不是公网，只给不分站点的插件用（例如通用网页）。

cookie 用 `Cookie::new(值, 主机)` 包起来，再经 `Request::cookie` 带上：跟跳转时，只有主机在这份名单里的那几跳带它，别的跳不带。`Cookie` 的 `Debug` 不打印值。

## 测试

测试要连本机的假服务器，而正式的策略不连 127.0.0.1。地址策略是一个值：测试自己拼一个 `Policy`，换掉查域名的函数和「算不算公网」的判断，用 `Client::with_policy` 交进去。

```rust
use std::net::IpAddr;
use usuyo_plugin_kit::http::Policy;

fn local_policy() -> Policy {
    Policy {
        lookup: |_, _| Err(std::io::Error::other("no DNS in tests")),
        public: |ip| ip == IpAddr::from([127, 0, 0, 1]),
    }
}
```

`Client::new` 和 `Policy::strict()` 总是严格的；没有环境变量、命令行参数或 feature 能把正式运行的检查放松。测试只连本机和固定的查域名表，不连真的网站。
