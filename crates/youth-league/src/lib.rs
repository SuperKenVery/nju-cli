use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://tuanwei.nju.edu.cn/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArticleSection {
    LatestDynamics,
    Announcements,
}

impl ArticleSection {
    pub fn title(self) -> &'static str {
        match self {
            ArticleSection::LatestDynamics => "最新动态",
            ArticleSection::Announcements => "公告通知",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            ArticleSection::LatestDynamics => "latest-dynamics",
            ArticleSection::Announcements => "announcements",
        }
    }

    fn list_path(self) -> &'static str {
        match self {
            ArticleSection::LatestDynamics => "24480",
            ArticleSection::Announcements => "ggtz",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticlePage {
    pub section: ArticleSection,
    pub page_index: u64,
    pub page_size: Option<u64>,
    pub total: Option<u64>,
    pub total_pages: Option<u64>,
    pub articles: Vec<Article>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub id: u64,
    pub title: String,
    pub publish_date: String,
    pub url: String,
}

pub async fn get_articles(
    client: &common::Client,
    section: ArticleSection,
    page_index: u64,
) -> Result<ArticlePage> {
    let url = list_url(section, page_index)?;
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to request youth league {} list", section.title()))?
        .error_for_status()
        .with_context(|| {
            format!(
                "youth league {} list returned an error status",
                section.title()
            )
        })?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read youth league {} list body", section.title()))?;

    parse_article_list(section, page_index, page_url.as_str(), &html)
}

pub async fn read_article(client: &common::Client, url: &str) -> Result<String> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid youth league site base URL")?
        .join(url)
        .with_context(|| format!("invalid youth league article URL: {url}"))?;
    let response = client
        .get(url.as_str())
        .send()
        .await
        .with_context(|| format!("failed to request youth league article: {url}"))?
        .error_for_status()
        .with_context(|| format!("youth league article returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read youth league article body: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

fn list_url(section: ArticleSection, page_index: u64) -> Result<reqwest::Url> {
    if page_index == 0 {
        return Err(anyhow!("page index starts from 1"));
    }

    let path = if page_index == 1 {
        format!("{}/list.htm", section.list_path())
    } else {
        format!("{}/list{page_index}.htm", section.list_path())
    };

    reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid youth league site base URL")?
        .join(&path)
        .with_context(|| format!("invalid youth league list path: {path}"))
}

fn parse_article_list(
    section: ArticleSection,
    page_index: u64,
    page_url: &str,
    html: &str,
) -> Result<ArticlePage> {
    let document = Html::parse_document(html);
    let item_selector = selector("ul.news_list li.news")?;
    let title_selector = selector(".news_title a")?;
    let date_selector = selector(".news_meta")?;
    let page_size_selector = selector(".per_count")?;
    let total_selector = selector("em.all_count")?;
    let total_pages_selector = selector(".pages .all_pages")?;
    let base_url =
        reqwest::Url::parse(page_url).with_context(|| format!("invalid page URL: {page_url}"))?;
    let mut articles = Vec::new();

    for item in document.select(&item_selector) {
        let Some(link) = item.select(&title_selector).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };

        let url = base_url
            .join(href)
            .with_context(|| format!("invalid article URL in youth league list: {href}"))?;
        let title = link
            .value()
            .attr("title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| text_content(link.text()));
        let publish_date = item
            .select(&date_selector)
            .next()
            .map(|date| text_content(date.text()))
            .unwrap_or_default();

        articles.push(Article {
            id: article_id(&url),
            title,
            publish_date,
            url: url.to_string(),
        });
    }

    Ok(ArticlePage {
        section,
        page_index,
        page_size: first_u64(&document, &page_size_selector),
        total: first_u64(&document, &total_selector),
        total_pages: first_u64(&document, &total_pages_selector),
        articles,
    })
}

fn article_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let article_selector = selector(".article")?;
    let article_html = document
        .select(&article_selector)
        .next()
        .map(|article| article.html())
        .unwrap_or_else(|| html.to_string());
    let mut markdown = common::html_to_markdown_with_base_url(&article_html, page_url)?;
    let pdf_urls = extract_pdf_urls(html, page_url)?;

    for pdf_url in pdf_urls {
        if !markdown.contains(&pdf_url) {
            if !markdown.ends_with('\n') {
                markdown.push('\n');
            }
            markdown.push_str(&format!("\nPDF 文件：<{pdf_url}>\n"));
        }
    }

    Ok(markdown)
}

fn extract_pdf_urls(html: &str, page_url: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let base_url =
        reqwest::Url::parse(page_url).with_context(|| format!("invalid page URL: {page_url}"))?;
    let pdf_player_selector = selector(".wp_pdf_player")?;
    let mut urls = Vec::new();

    for element in document.select(&pdf_player_selector) {
        if let Some(pdf_src) = element.value().attr("pdfsrc") {
            push_unique_url(&mut urls, &base_url, pdf_src);
        }

        if let Some(src) = element.value().attr("src") {
            if let Ok(viewer_url) = base_url.join(src) {
                if let Some((_, file)) = viewer_url.query_pairs().find(|(name, _)| name == "file") {
                    push_unique_url(&mut urls, &base_url, file.as_ref());
                }
            }
        }
    }

    Ok(urls)
}

fn push_unique_url(urls: &mut Vec<String>, base_url: &reqwest::Url, url: &str) {
    if let Ok(url) = base_url.join(url) {
        let url = url.to_string();
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn article_id(url: &reqwest::Url) -> u64 {
    if let Some(id) = internal_article_id(url) {
        return id;
    }

    0x8000_0000_0000_0000 | stable_hash(url.as_str())
}

fn internal_article_id(url: &reqwest::Url) -> Option<u64> {
    for segment in url.path_segments()? {
        if let Some((_, id)) = segment.rsplit_once('a') {
            let column = segment.split_once('a')?.0;
            if column.len() > 1
                && column.starts_with('c')
                && column[1..].bytes().all(|byte| byte.is_ascii_digit())
                && !id.is_empty()
                && id.bytes().all(|byte| byte.is_ascii_digit())
            {
                return id.parse().ok();
            }
        }
    }

    None
}

fn stable_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash & 0x7fff_ffff_ffff_ffff
}

fn first_u64(document: &Html, selector: &Selector) -> Option<u64> {
    document
        .select(selector)
        .next()
        .map(|element| text_content(element.text()))
        .and_then(|text| text.parse().ok())
}

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<Vec<_>>().join("").trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector).map_err(|error| anyhow!("invalid CSS selector {selector}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_article_list() {
        let html = r#"
            <ul class="news_list list2">
                <li class="news n1 clearfix">
                    <span class="news_title"><a href='/bb/46/c24691a834374/page.htm' title='关于做好南京大学第十次研究生代表大会代表选举工作的通知'>标题被截断</a></span>
                    <span class="news_meta">2026-06-01</span>
                </li>
            </ul>
            <span class="per_page">每页&nbsp;<em class="per_count">14</em>&nbsp;记录&nbsp;</span>
            <span class="all_count">总共&nbsp;<em class="all_count">1403</em>&nbsp;记录&nbsp;</span>
            <span class="pages">页码&nbsp;<em class="curr_page">1</em>/<em class="all_pages">101</em></span>
        "#;

        let page = parse_article_list(
            ArticleSection::Announcements,
            1,
            "https://tuanwei.nju.edu.cn/ggtz/list.htm",
            html,
        )
        .unwrap();

        assert_eq!(page.page_size, Some(14));
        assert_eq!(page.total, Some(1403));
        assert_eq!(page.total_pages, Some(101));
        assert_eq!(page.articles.len(), 1);
        assert_eq!(page.articles[0].id, 834374);
        assert_eq!(
            page.articles[0].title,
            "关于做好南京大学第十次研究生代表大会代表选举工作的通知"
        );
        assert_eq!(page.articles[0].publish_date, "2026-06-01");
        assert_eq!(
            page.articles[0].url,
            "https://tuanwei.nju.edu.cn/bb/46/c24691a834374/page.htm"
        );
    }

    #[test]
    fn keeps_pdf_player_urls_in_markdown() {
        let html = r#"
            <div class="article">
                <h1 class="arti_title">通知</h1>
                <div class="wp_articlecontent">
                    <iframe class="wp_pdf_player" src="/_js/_portletPlugs/swfPlayer/pdfjs22228/web/viewer.html?file=/_upload/article/files/f1/bc/a.pdf"></iframe>
                    <span class="wp_pdf_player" pdfsrc="/_upload/article/files/f1/bc/b.pdf"></span>
                </div>
            </div>
        "#;

        let markdown = article_html_to_markdown(
            html,
            "https://tuanwei.nju.edu.cn/bb/46/c24691a834374/page.htm",
        )
        .unwrap();

        assert!(markdown.contains("https://tuanwei.nju.edu.cn/_upload/article/files/f1/bc/a.pdf"));
        assert!(markdown.contains("https://tuanwei.nju.edu.cn/_upload/article/files/f1/bc/b.pdf"));
    }

    #[test]
    fn parses_article_id_only_from_column_segment() {
        let url =
            reqwest::Url::parse("https://tuanwei.nju.edu.cn/a9/be/c24480a829886/page.htm").unwrap();

        assert_eq!(internal_article_id(&url), Some(829886));
    }
}
