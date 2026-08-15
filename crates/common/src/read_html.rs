use anyhow::{Context, Result};
use html_to_markdown_rs::convert;
use regex::{Captures, Regex};
use reqwest::Url;
use std::sync::LazyLock;

/// 将 HTML 文本转换为 Markdown。
pub fn html_to_markdown(html: &str) -> Result<String> {
    let result = convert(html, None).context("failed to convert HTML to Markdown")?;

    result.content.context("No content in converted markdown")
}

/// 将 HTML 文本转换为 Markdown，并使用 `base_url` 补全 Markdown 链接中的相对 URL。
pub fn html_to_markdown_with_base_url(html: &str, base_url: &str) -> Result<String> {
    let markdown = html_to_markdown(html)?;
    let base_url = Url::parse(base_url).with_context(|| format!("invalid base URL: {base_url}"))?;

    Ok(absolutize_markdown_urls(&markdown, &base_url))
}

/// 使用调用方提供的 HTTP client 读取 HTML 页面，并转换为 Markdown。
pub async fn read_html_page(client: &crate::Client, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to request HTML page: {url}"))?
        .error_for_status()
        .with_context(|| format!("HTML page returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read HTML page body: {url}"))?;

    html_to_markdown_with_base_url(&html, page_url.as_str())
}

fn absolutize_markdown_urls(markdown: &str, base_url: &Url) -> String {
    static LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"(?<prefix>!?\[[^\]]*\]\()(?:(?<open><)(?<wrapped_url>[^>\r\n]+)(?<close>>)|(?<url>[^\s)\r\n]+))(?<title>\s+[^)\r\n]+)?(?<suffix>\))"#,
        )
        .expect("valid markdown link regex")
    });

    LINK_RE
        .replace_all(markdown, |captures: &Captures<'_>| {
            let url = captures
                .name("wrapped_url")
                .or_else(|| captures.name("url"))
                .expect("url capture exists")
                .as_str();
            let absolute_url = absolutize_url(url, base_url);

            format!(
                "{}{}{}{}{}{}",
                &captures["prefix"],
                captures.name("open").map_or("", |m| m.as_str()),
                absolute_url,
                captures.name("close").map_or("", |m| m.as_str()),
                captures.name("title").map_or("", |m| m.as_str()),
                &captures["suffix"]
            )
        })
        .into_owned()
}

fn absolutize_url(url: &str, base_url: &Url) -> String {
    base_url
        .join(url)
        .map_or_else(|_| url.to_string(), |url| url.to_string())
}
