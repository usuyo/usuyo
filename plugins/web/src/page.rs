//! 取一个网页：原样存下 HTML，抽出正文和元数据，按 `--want` 下载正文里的图片。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use dom_smoothie::{Article, Config, Readability, TextMode};
use encoding_rs::Encoding;
use serde_json::json;
use ureq::ResponseExt;
use usuyo_plugin::protocol::{
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

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .timeout_connect(Some(Duration::from_secs(15)))
        .max_redirects(MAX_REDIRECTS)
        .user_agent(USER_AGENT)
        .http_status_as_error(false)
        .build()
        .into()
}

pub(crate) fn fetch(
    input: &str,
    out: &Path,
    want: Option<&Vec<OutputKind>>,
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

    let agent = agent(Duration::from_secs(60));
    let fetched_at = Rfc3339::from_timestamp(jiff::Timestamp::now());
    let mut response = agent
        .get(input)
        .header("Accept", "text/html,application/xhtml+xml;q=0.9,*/*;q=0.5")
        .call()
        .map_err(network)?;
    let status = response.status().as_u16();
    check_status(status, response.headers().get("retry-after"))?;
    let final_url = response.get_uri().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = response
        .body_mut()
        .with_config()
        .limit(PAGE_BYTES)
        .read_to_vec()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(_) => fail(
                ErrorCode::UnsupportedInput,
                format!("page is larger than {PAGE_BYTES} bytes"),
            ),
            other => network(other),
        })?;
    if !is_html(&content_type, &bytes) {
        return Err(fail(
            ErrorCode::UnsupportedInput,
            format!("not an HTML page (content-type `{content_type}`)"),
        ));
    }

    let html = decode(&content_type, &bytes);
    let base = url::Url::parse(&final_url).with_context(|| format!("final URL `{final_url}`"))?;
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
            reasons.push("没有找到正文");
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
        let downloaded = download_images(&sources, &base, out, &mut failed_images)?;
        if downloaded.is_empty() && !sources.is_empty() {
            output.missing.push(OutputKind::Images);
            reasons.push("正文里的图片没有下载下来");
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
            "finalUrl": final_url,
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

fn network(e: ureq::Error) -> anyhow::Error {
    let code = match &e {
        ureq::Error::TooManyRedirects => ErrorCode::UnsupportedInput,
        ureq::Error::BadUri(_) => ErrorCode::UnsupportedInput,
        _ => ErrorCode::Network,
    };
    fail(code, e.to_string())
}

fn check_status(status: u16, retry_after: Option<&ureq::http::HeaderValue>) -> Result<()> {
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
            retry_after: retry_after
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse().ok()),
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

/// 一张张下载；单张失败或超出时间预算的记下来，不让整次失败。写盘失败才是错误。
fn download_images(
    sources: &[url::Url],
    page: &url::Url,
    out: &Path,
    failed: &mut Vec<String>,
) -> Result<Vec<(RelPath, String)>> {
    let agent = agent(IMAGE_TIMEOUT);
    let started = std::time::Instant::now();
    let mut saved = Vec::new();
    let mut total = 0u64;
    for url in sources {
        if started.elapsed() > IMAGES_BUDGET {
            failed.push(url.to_string());
            continue;
        }
        let got = agent
            .get(url.as_str())
            .header("Referer", page.as_str())
            .header("Accept", "image/*")
            .call()
            .ok()
            .filter(|r| r.status().is_success())
            .and_then(|mut r| {
                let mime = r.headers().get("content-type")?.to_str().ok()?.to_owned();
                let ext = image_extension(&mime)?;
                let bytes = r
                    .body_mut()
                    .with_config()
                    .limit(IMAGE_BYTES)
                    .read_to_vec()
                    .ok()?;
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
    use super::*;

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
