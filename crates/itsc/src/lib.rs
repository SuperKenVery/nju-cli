use std::collections::{HashSet, VecDeque};

use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://itsc.nju.edu.cn/";
const SITE_HOST: &str = "itsc.nju.edu.cn";
const WEB_VPN_SITE_HOST: &str = "itsc-nju-edu-cn.atrust.nju.edu.cn";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Section {
    Services,
    LicensedSoftware,
}

impl Section {
    pub fn title(self) -> &'static str {
        match self {
            Self::Services => "信息化中心服务说明",
            Self::LicensedSoftware => "正版软件安装教程",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Services => "services",
            Self::LicensedSoftware => "licensed-software",
        }
    }

    fn url(self) -> &'static str {
        match self {
            Self::Services => "https://itsc.nju.edu.cn/21426/list.htm",
            Self::LicensedSoftware => "https://itsc.nju.edu.cn/zbrj/list.htm",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageKind {
    Column,
    Article,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub kind: PageKind,
}

/// 列出 ITSC 栏目页面。
///
/// `recursive` 为 true 时，会从栏目页继续抓取同域名下的栏目页和文章页，适合
/// 正版软件这种正文里链接到大量子教程的页面。
pub async fn list_pages(
    client: &common::Client,
    section: Section,
    recursive: bool,
    max_pages: usize,
) -> Result<Vec<Page>> {
    let max_pages = max_pages.max(1);
    let start_url = reqwest::Url::parse(section.url())
        .with_context(|| format!("invalid ITSC section URL: {}", section.url()))?;
    let mut queue = VecDeque::from([start_url]);
    let mut visited = HashSet::<String>::new();
    let mut pages = Vec::new();
    let mut page_urls = HashSet::new();
    let mut page_ids = HashSet::new();

    while let Some(url) = queue.pop_front() {
        let normalized = normalize_url(url).to_string();
        if !visited.insert(normalized.clone()) {
            continue;
        }
        if visited.len() > max_pages {
            break;
        }

        let response = client
            .get(&normalized)
            .send()
            .await
            .with_context(|| format!("failed to request ITSC page: {normalized}"))?
            .error_for_status()
            .with_context(|| format!("ITSC page returned an error status: {normalized}"))?;
        let page_url = normalize_url(response.url().clone());
        let html = response
            .text()
            .await
            .with_context(|| format!("failed to read ITSC page body: {page_url}"))?;
        let document = Html::parse_document(&html);

        if page_urls.insert(page_url.to_string()) {
            let mut page = parse_page(&document, page_url.as_str())?;
            if !page_ids.insert(page.id) {
                page.id = stable_id(&page.url);
                page_ids.insert(page.id);
            }
            pages.push(page);
        }

        if !recursive && page_url != normalize_url(reqwest::Url::parse(section.url())?) {
            continue;
        }

        for link in extract_follow_links(&document, page_url.as_str(), recursive)? {
            let link = normalize_url(link);
            if !visited.contains(link.as_str()) {
                queue.push_back(link);
            }
        }
    }

    Ok(pages)
}

/// 读取 ITSC 页面正文并转换为 Markdown。
pub async fn read_page(client: &common::Client, url: &str) -> Result<String> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid ITSC site base URL")?
        .join(url)
        .with_context(|| format!("invalid ITSC page URL: {url}"))?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request ITSC page: {url}"))?
        .error_for_status()
        .with_context(|| format!("ITSC page returned an error status: {url}"))?;
    let page_url = normalize_url(response.url().clone());
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read ITSC page body: {url}"))?;

    page_html_to_markdown(&html, page_url.as_str())
}

fn parse_page(document: &Html, page_url: &str) -> Result<Page> {
    let url = reqwest::Url::parse(page_url)
        .with_context(|| format!("invalid ITSC page URL: {page_url}"))?;
    let title = page_title(document)?;
    let id = visit_count_id(document)
        .or_else(|| id_from_page_url(&url))
        .unwrap_or_else(|| stable_id(page_url));
    let kind = if url.path().ends_with("/page.htm") {
        PageKind::Article
    } else {
        PageKind::Column
    };

    Ok(Page {
        id,
        title,
        url: url.to_string(),
        kind,
    })
}

fn extract_follow_links(
    document: &Html,
    page_url: &str,
    recursive: bool,
) -> Result<Vec<reqwest::Url>> {
    let base_url = reqwest::Url::parse(page_url)
        .with_context(|| format!("invalid ITSC page URL: {page_url}"))?;
    let selector = if recursive {
        selector(".col_menu_con a[href], .wp_articlecontent a[href], .col_news_con a[href]")?
    } else {
        selector(".col_menu_con a[href]")?
    };
    let mut urls = Vec::new();
    let mut seen = HashSet::new();

    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let Ok(url) = base_url.join(href) else {
            continue;
        };
        let url = normalize_url(url);
        if !is_itsc_content_url(&url) {
            continue;
        }
        if seen.insert(url.to_string()) {
            urls.push(url);
        }
    }

    Ok(urls)
}

fn page_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let content_selector = selector(".wp_articlecontent, .article, .col_news_con")?;
    let content_html = document
        .select(&content_selector)
        .next()
        .map(|content| content.html())
        .unwrap_or_else(|| html.to_string());
    let mut markdown = common::html_to_markdown_with_base_url(&content_html, page_url)?;
    let title = page_title(&document).ok();

    if let Some(title) = title {
        markdown = format!("# {title}\n\n{}", markdown.trim_start());
    }

    Ok(markdown)
}

fn page_title(document: &Html) -> Result<String> {
    let selector = selector(".col_title h2, .arti_title, title")?;

    document
        .select(&selector)
        .find_map(|element| {
            let title = text_content(element.text());
            (!title.is_empty()).then_some(title)
        })
        .ok_or_else(|| anyhow!("failed to find ITSC page title"))
}

fn visit_count_id(document: &Html) -> Option<u64> {
    let selector = selector(r#"img[src*="_visitcount"]"#).ok()?;

    for element in document.select(&selector) {
        let src = element.value().attr("src")?;
        let url = reqwest::Url::parse(SITE_BASE_URL).ok()?.join(src).ok()?;
        for (name, value) in url.query_pairs() {
            if name == "articleId" || name == "columnId" {
                if let Ok(id) = value.parse() {
                    return Some(id);
                }
            }
        }
    }

    None
}

fn id_from_page_url(url: &reqwest::Url) -> Option<u64> {
    let path = url.path();

    if path.ends_with("/list.htm") {
        return path
            .trim_end_matches("/list.htm")
            .rsplit('/')
            .next()
            .and_then(|segment| segment.parse().ok());
    }

    if path.ends_with("/page.htm") {
        return path
            .trim_end_matches("/page.htm")
            .rsplit('/')
            .next()
            .and_then(|segment| segment.strip_prefix('a').unwrap_or(segment).parse().ok());
    }

    None
}

fn is_itsc_content_url(url: &reqwest::Url) -> bool {
    if !matches!(url.host_str(), Some(SITE_HOST | WEB_VPN_SITE_HOST)) {
        return false;
    }

    let path = url.path();
    path.ends_with("/list.htm") || path.ends_with("/page.htm")
}

fn normalize_url(mut url: reqwest::Url) -> reqwest::Url {
    url.set_fragment(None);
    url
}

fn stable_id(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<String>().trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector).map_err(|error| anyhow!("invalid CSS selector {selector}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_visit_count_column_id() {
        let html = r#"
            <h2>微软Office</h2>
            <img src="/_visitcount?siteId=302&type=2&columnId=46454" />
        "#;
        let document = Html::parse_document(html);

        assert_eq!(visit_count_id(&document), Some(46454));
    }

    #[test]
    fn follows_content_subpages_when_recursive() {
        let html = r#"
            <div class="wp_articlecontent">
                <a href="/Office/list.htm">Office</a>
                <a href="https://download.nju.edu.cn/file.zip">下载</a>
            </div>
        "#;
        let document = Html::parse_document(html);
        let links =
            extract_follow_links(&document, "https://itsc.nju.edu.cn/zbrj/list.htm", true).unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].as_str(), "https://itsc.nju.edu.cn/Office/list.htm");
    }

    #[test]
    fn follows_content_subpages_from_web_vpn_response() {
        let document = Html::parse_document(
            r#"<div class="wp_articlecontent"><a href="/Office/list.htm">Office</a></div>"#,
        );
        let links = extract_follow_links(
            &document,
            "https://itsc-nju-edu-cn.atrust.nju.edu.cn/zbrj/list.htm",
            true,
        )
        .unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(
            links[0].as_str(),
            "https://itsc-nju-edu-cn.atrust.nju.edu.cn/Office/list.htm"
        );
    }

    #[test]
    fn normalizes_only_the_fragment() {
        let url = reqwest::Url::parse(
            "http://itsc-nju-edu-cn.atrust.nju.edu.cn/Office/list.htm?q=1#section",
        )
        .unwrap();

        assert_eq!(
            normalize_url(url).as_str(),
            "http://itsc-nju-edu-cn.atrust.nju.edu.cn/Office/list.htm?q=1"
        );
    }
}
