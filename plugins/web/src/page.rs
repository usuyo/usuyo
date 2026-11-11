//! 取一个网页：原样存下 HTML，抽出正文和元数据，按 `--want` 下载正文里的图片。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use dom_smoothie::{Article, Config, Readability, TextMode};
use encoding_rs::Encoding;
use serde_json::json;
use usuyo_plugin_kit::http::{self, Client, Hosts, Policy};
use usuyo_plugin_kit::protocol::{
    Access, Author, CallFailure, Completeness, ErrorCode, Evidence, FetchOutput, ItemKind,
    NotFoundReason, OutputKind, RelPath, Rfc3339, TextFormat,
};

use crate::{Fail, fail, web_identity};

/// 普通桌面浏览器的 UA：有的站点对不认识的 UA 只给空壳页面。
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
const PAGE_BYTES: u64 = 16 << 20;
const IMAGE_BYTES: u64 = 20 << 20;
const IMAGES_TOTAL_BYTES: u64 = 256 << 20;
const MAX_IMAGES: usize = 64;
const MAX_REDIRECTS: u32 = 10;
const PAGE_TIMEOUT: Duration = Duration::from_secs(60);
/// 单张图片的总时长，和所有图片加起来的时长：图片慢不能拖到宿主的 fetch 超时，丢掉已经抽好的正文。
const IMAGE_TIMEOUT: Duration = Duration::from_secs(15);
const IMAGES_BUDGET: Duration = Duration::from_secs(90);

/// 本插件能给的输出，和清单里的 `outputs` 一致。
const OUTPUTS: [OutputKind; 4] = [
    OutputKind::Text,
    OutputKind::Html,
    OutputKind::Images,
    OutputKind::Meta,
];

/// 要取的页面取不了时报的错。地址不是公网算「不是这个插件取的链接」（unsupported-input）：
/// 换别的获取方式也许能取，重试没用。
fn page_failure(e: http::Error) -> anyhow::Error {
    match e {
        http::Error::NotPublic { why, first: true } => Fail(CallFailure {
            hint: Some(
                "这个链接指向本机或内网地址，没有去取。要收下自己内网里的网页，在「网页」获取方式的设置里打开「允许取内网网页」。"
                    .into(),
            ),
            ..CallFailure::new(ErrorCode::UnsupportedInput, why)
        })
        .into(),
        http::Error::NotPublic { why, first: false } => fail(
            ErrorCode::UnsupportedInput,
            format!("redirect refused: {why}"),
        ),
        http::Error::TooLarge(limit) => fail(
            ErrorCode::UnsupportedInput,
            format!("page is larger than {limit} bytes"),
        ),
        e @ (http::Error::Invalid(_)
        | http::Error::TooManyRedirects(_)
        | http::Error::BadLocation(_)) => fail(ErrorCode::UnsupportedInput, e.to_string()),
        e @ (http::Error::Lookup(_) | http::Error::Timeout | http::Error::Network(_)) => {
            fail(ErrorCode::Network, e.to_string())
        }
        // 不限主机、不带 cookie，走不到这两种
        e @ (http::Error::NotAllowed { .. } | http::Error::CookieHeader) => {
            fail(ErrorCode::Internal, e.to_string())
        }
    }
}

/// 取一个网页。`private_page` 来自插件设置，只放开这条链接本身，图片和跳转目标不放开。
pub(crate) fn fetch(
    input: &str,
    out: &Path,
    want: Option<&Vec<OutputKind>>,
    private_page: bool,
    policy: &Policy,
) -> Result<FetchOutput> {
    let identity = web_identity(input).ok_or_else(|| {
        fail(
            ErrorCode::UnsupportedInput,
            format!("`{input}` is not an http(s) URL"),
        )
    })?;
    let wanted: BTreeSet<OutputKind> =
        want.map_or_else(|| OUTPUTS.into(), |w| w.iter().copied().collect());
    if let Some(other) = wanted.iter().find(|w| !OUTPUTS.contains(w)) {
        return Err(fail(
            ErrorCode::InvalidArgs,
            format!("cannot give `{other}`"),
        ));
    }

    let client = Client::new(USER_AGENT, Hosts::any()).with_policy(*policy);
    let start = url::Url::parse(input).context("parse input URL")?;
    let fetched_at = Rfc3339::from_timestamp(jiff::Timestamp::now());
    let mut response = client
        .get(&start)
        .header("Accept", "text/html,application/xhtml+xml;q=0.9,*/*;q=0.5")
        .timeout(PAGE_TIMEOUT)
        .follow_redirects(MAX_REDIRECTS)
        .allow_private_first_hop(private_page)
        .send()
        .map_err(page_failure)?;
    let base = response.url().clone();
    let status = response.status();
    check_status(status, response.header("retry-after"))?;
    let content_type = response
        .header("content-type")
        .unwrap_or_default()
        .to_owned();
    let bytes = response.read_to_vec(PAGE_BYTES).map_err(page_failure)?;
    if !is_html(&content_type, &bytes) {
        return Err(fail(
            ErrorCode::UnsupportedInput,
            format!("not an HTML page (content-type `{content_type}`)"),
        ));
    }

    let html = decode(&content_type, &bytes);
    let canonical = canonical_link(&html, &base);
    let mut reader = Readability::new(
        html,
        Some(base.as_str()),
        Some(Config {
            text_mode: TextMode::Formatted,
            ..Config::default()
        }),
    )
    .context("prepare readability")?;
    let metadata = reader.get_article_metadata(reader.parse_json_ld());
    let article: Option<Article> = reader.parse().ok();

    let mut output = FetchOutput {
        identity: Some(identity),
        kind: ItemKind::Article,
        title: None,
        author: None,
        published_at: None,
        text: None,
        text_format: None,
        html: None,
        images: Vec::new(),
        media: Vec::new(),
        platform_tags: Vec::new(),
        comments: None,
        completeness: Completeness::Full,
        missing: Vec::new(),
        missing_reason: None,
        evidence: Evidence {
            fetched_at,
            access: Access::PublicWeb,
            site: None,
            upstream: None,
            status: Some(status),
        },
        credentials_update: None,
    };
    let mut reasons = Vec::new();

    if wanted.contains(&OutputKind::Html) {
        fs::write(out.join("page.html"), &bytes).context("write page.html")?;
        output.html = Some(rel("page.html")?);
    }
    if wanted.contains(&OutputKind::Text) {
        output.text = article
            .as_ref()
            .map(|a| a.text_content.trim().to_owned())
            .filter(|t| !t.is_empty());
        output.text_format = output.text.as_ref().map(|_| TextFormat::Plain);
        if output.text.is_none() {
            output.missing.push(OutputKind::Text);
            reasons.push("没有找到正文".to_owned());
        }
    }

    let title = article
        .as_ref()
        .map(|a| a.title.trim().to_owned())
        .filter(|t| !t.is_empty())
        .or_else(|| Some(metadata.title.trim().to_owned()).filter(|t| !t.is_empty()));
    let byline = article
        .as_ref()
        .and_then(|a| a.byline.clone())
        .or(metadata.byline.clone());
    let published_raw = article
        .as_ref()
        .and_then(|a| a.published_time.clone())
        .or(metadata.published_time.clone());
    let lead_image = article
        .as_ref()
        .and_then(|a| a.image.clone())
        .or(metadata.image.clone())
        .and_then(|i| base.join(&i).ok())
        .map(String::from);

    let mut image_records = Vec::new();
    let mut failed_images = Vec::new();
    if wanted.contains(&OutputKind::Images) {
        let sources = article
            .as_ref()
            .map(|a| image_sources(&a.content, &base))
            .unwrap_or_default();
        let downloaded = download_images(&sources, &base, out, &mut failed_images, &client)?;
        if downloaded.is_empty() && !sources.is_empty() {
            output.missing.push(OutputKind::Images);
            reasons.push("正文里的图片没有下载下来".to_owned());
        } else if !failed_images.is_empty() {
            // 拿到了一些、少了几张：这一类不算缺，但要如实说。
            reasons.push(format!(
                "正文里有 {} 张图片没有下载下来",
                failed_images.len()
            ));
        }
        for (path, url) in downloaded {
            image_records.push(json!({"path": path.as_str(), "url": url}));
            output.images.push(path);
        }
    }

    if wanted.contains(&OutputKind::Meta) {
        output.title = title.clone();
        output.author = byline
            .as_ref()
            .map(|b| b.trim().to_owned())
            .filter(|b| !b.is_empty())
            .map(|name| Author {
                name,
                platform_id: None,
                url: None,
            });
        output.published_at = published_raw
            .as_ref()
            .and_then(|p| Rfc3339::try_from(p.clone()).ok());
        // 协议的输出没有描述、题图、最终链接这几栏，写进 meta.json，随快照留在 raw/
        let meta = json!({
            "url": input,
            "finalUrl": base.as_str(),
            "canonicalUrl": canonical,
            "contentType": content_type,
            "title": title,
            "byline": byline,
            "published": published_raw,
            "description": article.as_ref().and_then(|a| a.excerpt.clone()).or(metadata.excerpt.clone()),
            "leadImage": lead_image,
            "siteName": article.as_ref().and_then(|a| a.site_name.clone()).or(metadata.site_name.clone()),
            "lang": article.as_ref().and_then(|a| a.lang.clone()).or(metadata.lang.clone()),
            "images": image_records,
            "failedImages": failed_images,
        });
        fs::write(out.join("meta.json"), serde_json::to_vec_pretty(&meta)?)
            .context("write meta.json")?;
    }

    output.completeness = if output.missing.is_empty() {
        Completeness::Full
    } else {
        Completeness::Partial
    };
    if !reasons.is_empty() {
        output.missing_reason = Some(reasons.join("；"));
    }
    Ok(output)
}

fn rel(path: &str) -> Result<RelPath> {
    RelPath::try_from(path).map_err(anyhow::Error::msg)
}

fn check_status(status: u16, retry_after: Option<&str>) -> Result<()> {
    let failure = |code: ErrorCode| CallFailure::new(code, format!("HTTP {status}"));
    let f = match status {
        200..=299 => return Ok(()),
        // 公开网页要登录才看得到，对不登录的插件就是「当前身份看不到」
        401 | 403 => CallFailure {
            reason: Some(NotFoundReason::Private),
            ..failure(ErrorCode::NotFound)
        },
        404 => failure(ErrorCode::NotFound),
        410 => CallFailure {
            reason: Some(NotFoundReason::Deleted),
            ..failure(ErrorCode::NotFound)
        },
        429 => CallFailure {
            retry_after: retry_after.and_then(|v| v.trim().parse().ok()),
            ..failure(ErrorCode::RateLimited)
        },
        500..=599 => failure(ErrorCode::Network),
        _ => failure(ErrorCode::UnsupportedInput),
    };
    Err(Fail(f).into())
}

fn is_html(content_type: &str, bytes: &[u8]) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match mime.as_str() {
        "text/html" | "application/xhtml+xml" => true,
        "" => {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
            head.contains("<html") || head.contains("<!doctype html")
        }
        _ => false,
    }
}

/// 按 Content-Type 的 charset，没有就看页面开头的 `<meta charset>`，再没有按 UTF-8；BOM 优先。
fn decode(content_type: &str, bytes: &[u8]) -> String {
    let declared = charset_param(content_type).or_else(|| {
        let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_ascii_lowercase();
        let at = head.find("charset=")? + "charset=".len();
        let rest = head[at..].trim_start_matches(['"', '\'']);
        let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))?;
        Some(rest[..end].to_owned())
    });
    let encoding = declared
        .and_then(|label| Encoding::for_label(label.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    let (text, _, _) = encoding.decode(bytes);
    text.into_owned()
}

fn charset_param(content_type: &str) -> Option<String> {
    content_type.split(';').skip(1).find_map(|p| {
        let (k, v) = p.split_once('=')?;
        (k.trim().eq_ignore_ascii_case("charset")).then(|| v.trim().trim_matches('"').to_owned())
    })
}

fn canonical_link(html: &str, base: &url::Url) -> Option<String> {
    let doc = dom_query::Document::from(html);
    let href = doc.select("link[rel=canonical]").attr("href")?;
    base.join(href.as_ref()).ok().map(String::from)
}

/// 正文里 `<img>` 的地址，去重，只要 http(s)。
fn image_sources(content: &str, base: &url::Url) -> Vec<url::Url> {
    let doc = dom_query::Document::from(content);
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for img in doc.select("img").iter() {
        let Some(src) = img.attr("src") else { continue };
        let Ok(url) = base.join(src.trim()) else {
            continue;
        };
        if matches!(url.scheme(), "http" | "https") && seen.insert(url.to_string()) {
            out.push(url);
        }
        if out.len() == MAX_IMAGES {
            break;
        }
    }
    out
}

/// 一张张下载；单张失败、地址不是公网或超出时间预算的记下来，不让整次失败。写盘失败才是错误。
fn download_images(
    sources: &[url::Url],
    page: &url::Url,
    out: &Path,
    failed: &mut Vec<String>,
    client: &Client,
) -> Result<Vec<(RelPath, String)>> {
    let started = std::time::Instant::now();
    let mut saved = Vec::new();
    let mut total = 0u64;
    for url in sources {
        if started.elapsed() > IMAGES_BUDGET {
            failed.push(url.to_string());
            continue;
        }
        let mut request = client.get(url).header("Accept", "image/*");
        if let Some(referer) = image_referer(page, url) {
            request = request.header("Referer", &referer);
        }
        let got = request
            .timeout(IMAGE_TIMEOUT)
            .follow_redirects(MAX_REDIRECTS)
            .send()
            .ok()
            .filter(|r| (200..300).contains(&r.status()))
            .and_then(|mut r| {
                let ext = image_extension(r.header("content-type")?)?;
                let bytes = r.read_to_vec(IMAGE_BYTES).ok()?;
                Some((ext, bytes))
            });
        let Some((ext, bytes)) = got else {
            failed.push(url.to_string());
            continue;
        };
        total += bytes.len() as u64;
        if total > IMAGES_TOTAL_BYTES {
            failed.push(url.to_string());
            break;
        }
        if saved.is_empty() {
            fs::create_dir_all(out.join("images")).context("create images/")?;
        }
        let name = format!("images/{:03}.{ext}", saved.len() + 1);
        fs::write(out.join(&name), &bytes).with_context(|| format!("write {name}"))?;
        saved.push((rel(&name)?, url.to_string()));
    }
    Ok(saved)
}

/// 照浏览器跨源取图的默认做法：只带页面的源，不带路径和查询（分享链接的令牌常在查询里）；
/// 页面是 https、图片是 http 时什么都不带。跳转后的那几跳由 kit 决定带不带。
fn image_referer(page: &url::Url, image: &url::Url) -> Option<String> {
    if page.scheme() == "https" && image.scheme() == "http" {
        return None;
    }
    Some(format!("{}/", page.origin().ascii_serialization()))
}

fn image_extension(mime: &str) -> Option<&'static str> {
    let mime = mime.split(';').next()?.trim().to_ascii_lowercase();
    Some(match mime.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::net::SocketAddr;

    use super::*;

    /// 测试用的策略：只有 127.0.0.1 算「公网」，`site.test` 解析到它。
    /// 这样本机测试服务能当公网站点用，127.0.0.2 之类照样被拒。
    fn local_policy() -> Policy {
        fn lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
            match host {
                "site.test" => Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))]),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "no such host")),
            }
        }
        Policy {
            lookup,
            public: |ip| ip == std::net::IpAddr::from([127, 0, 0, 1]),
            cookies_over_http: false,
            ..Policy::strict()
        }
    }

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake";

    fn article(port: u16) -> String {
        let para = "三段式注水是一种常见的手冲做法。第一段用 40 克水闷蒸 30 秒，让咖啡粉充分排气，这一步决定了后面萃取是否均匀，水温控制在 92 度左右比较合适，粉的研磨度接近细砂糖。";
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>手冲</title></head><body><article>
<h1>手冲</h1><p>{para}</p>
<p><img src="/img/a.png"></p><p>{para}</p>
<p><img src="/img/r.png"></p><p>{para}</p>
<p><img src="http://127.0.0.2:{port}/img/c.png"></p><p>{para}</p>
<p><img src="/img/missing.png"></p><p>{para}</p>
</article></body></html>"#
        )
    }

    /// 每个请求的路径（不含查询）和带的 Referer。
    type Seen = std::sync::Arc<std::sync::Mutex<Vec<(String, Option<String>)>>>;

    /// 本机 HTTP 服务：`/hop/N` 跳 N 次到正文，`/img/r.png` 跳到 `/img/b.png`。
    fn serve() -> (u16, Seen) {
        use std::io::{BufRead, BufReader, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Seen::default();
        let log = seen.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let target = line.split_whitespace().nth(1).unwrap_or("/");
                let path = target.split('?').next().unwrap_or_default().to_owned();
                let mut referer = None;
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                        break;
                    }
                    if let Some(v) = header.to_ascii_lowercase().strip_prefix("referer:") {
                        referer = Some(v.trim().to_owned());
                    }
                }
                log.lock().unwrap().push((path.clone(), referer));
                let redirect = |to: String| ("302 Found", "text/plain", Vec::new(), to);
                let (status, ctype, body, location) = match path.as_str() {
                    "/hop/0" => (
                        "200 OK",
                        "text/html",
                        article(port).into_bytes(),
                        String::new(),
                    ),
                    p if p.starts_with("/hop/") => {
                        let n: u32 = p["/hop/".len()..].parse().unwrap();
                        redirect(format!("/hop/{}", n - 1))
                    }
                    "/img/a.png" | "/img/b.png" => {
                        ("200 OK", "image/png", PNG.to_vec(), String::new())
                    }
                    "/img/r.png" => redirect("/img/b.png".into()),
                    _ => (
                        "404 Not Found",
                        "text/html",
                        b"nope".to_vec(),
                        String::new(),
                    ),
                };
                let location = if location.is_empty() {
                    String::new()
                } else {
                    format!("Location: {location}\r\n")
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n{location}\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).ok();
                stream.write_all(&body).ok();
            }
        });
        (port, seen)
    }

    #[test]
    fn follows_redirects_and_downloads_images() {
        let (port, _) = serve();
        let out = tempfile::tempdir().unwrap();
        let input = format!("http://site.test:{port}/hop/3");
        let output = fetch(&input, out.path(), None, false, &local_policy()).unwrap();

        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(out.path().join("meta.json")).unwrap()).unwrap();
        assert_eq!(meta["url"], input);
        assert_eq!(meta["finalUrl"], format!("http://site.test:{port}/hop/0"));

        // a.png 直接取到，r.png 跟一次跳转取到 b.png
        let paths: Vec<&str> = output.images.iter().map(RelPath::as_str).collect();
        assert_eq!(paths, ["images/001.png", "images/002.png"]);
        for path in paths {
            assert_eq!(fs::read(out.path().join(path)).unwrap(), PNG);
        }
        let base = format!("http://site.test:{port}");
        assert_eq!(
            meta["images"],
            json!([
                {"path": "images/001.png", "url": format!("{base}/img/a.png")},
                {"path": "images/002.png", "url": format!("{base}/img/r.png")},
            ])
        );
        // 被策略拒的那张和 404 的那张记下来，不影响别的
        assert_eq!(
            meta["failedImages"],
            json!([
                format!("http://127.0.0.2:{port}/img/c.png"),
                format!("{base}/img/missing.png"),
            ])
        );
        // 图片这一类拿到了，所以不算缺；少的两张写在原因里
        assert_eq!(output.completeness, Completeness::Full);
        assert_eq!(
            output.missing_reason.as_deref(),
            Some("正文里有 2 张图片没有下载下来")
        );
    }

    #[test]
    fn redirect_limit_still_applies() {
        let (port, _) = serve();
        let out = tempfile::tempdir().unwrap();
        let policy = local_policy();
        let at_limit = format!("http://site.test:{port}/hop/{MAX_REDIRECTS}");
        assert!(
            fetch(
                &at_limit,
                out.path(),
                Some(&vec![OutputKind::Text]),
                false,
                &policy
            )
            .is_ok()
        );

        let over = format!("http://site.test:{port}/hop/{}", MAX_REDIRECTS + 1);
        let err = fetch(
            &over,
            out.path(),
            Some(&vec![OutputKind::Text]),
            false,
            &policy,
        )
        .unwrap_err();
        let Fail(failure) = err.downcast::<Fail>().unwrap();
        assert_eq!(failure.code, ErrorCode::UnsupportedInput);
        assert!(failure.message.contains("redirects"), "{}", failure.message);
    }

    /// 取图只带页面的源当 Referer：页面地址里的路径和分享令牌不发给图片主机。
    #[test]
    fn images_get_only_the_page_origin_as_referer() {
        let (port, seen) = serve();
        let out = tempfile::tempdir().unwrap();
        let input = format!("http://site.test:{port}/hop/0?share_token=s3cret");
        fetch(&input, out.path(), None, false, &local_policy()).unwrap();
        let log = seen.lock().unwrap().clone();
        let images: Vec<_> = log.iter().filter(|(p, _)| p.starts_with("/img/")).collect();
        assert!(!images.is_empty());
        let origin = format!("http://site.test:{port}/");
        for (path, referer) in images {
            assert_eq!(referer.as_deref(), Some(origin.as_str()), "{path}");
        }
    }

    #[test]
    fn referer_rule() {
        let url = |s: &str| url::Url::parse(s).unwrap();
        let page = url("https://a.example/post/1?token=t");
        assert_eq!(
            image_referer(&page, &url("https://cdn.example/x.png")).as_deref(),
            Some("https://a.example/")
        );
        assert_eq!(image_referer(&page, &url("http://cdn.example/x.png")), None);
        let plain = url("http://a.example:8080/post?token=t");
        assert_eq!(
            image_referer(&plain, &url("http://cdn.example/x.png")).as_deref(),
            Some("http://a.example:8080/")
        );
    }

    #[test]
    fn decodes_declared_charsets() {
        let (gbk, _, _) = encoding_rs::GBK
            .encode("<html><head><meta charset=\"gbk\"></head><body>手冲咖啡</body></html>");
        assert!(decode("text/html", &gbk).contains("手冲咖啡"));
        assert!(decode("text/html; charset=GBK", &gbk).contains("手冲咖啡"));
        assert!(decode("text/html", "<p>中文</p>".as_bytes()).contains("中文"));
    }

    #[test]
    fn html_detection() {
        assert!(is_html("text/html; charset=utf-8", b""));
        assert!(is_html("", b"<!DOCTYPE html><html>"));
        assert!(!is_html("application/pdf", b"%PDF"));
    }
}
