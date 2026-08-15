use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://yzb.nju.edu.cn/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArticleSection {
    MasterGuide,
    MasterNotifications,
    DoctoralGuide,
    DoctoralNotifications,
    SummerCampRecommendation,
    HongKongMacaoTaiwanGuide,
    HongKongMacaoTaiwanNotifications,
    PublicNotices,
    ScoreLines,
    AdmissionStatistics,
}

impl ArticleSection {
    pub const ALL: &'static [ArticleSection] = &[
        ArticleSection::MasterGuide,
        ArticleSection::MasterNotifications,
        ArticleSection::DoctoralGuide,
        ArticleSection::DoctoralNotifications,
        ArticleSection::SummerCampRecommendation,
        ArticleSection::HongKongMacaoTaiwanGuide,
        ArticleSection::HongKongMacaoTaiwanNotifications,
        ArticleSection::PublicNotices,
        ArticleSection::ScoreLines,
        ArticleSection::AdmissionStatistics,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::MasterGuide => "硕士招生：简章目录",
            Self::MasterNotifications => "硕士招生：硕士最新通知",
            Self::DoctoralGuide => "博士招生：简章目录",
            Self::DoctoralNotifications => "博士招生：博士最新通知",
            Self::SummerCampRecommendation => "夏令营/推免：最新公告",
            Self::HongKongMacaoTaiwanGuide => "港澳台招生：简章目录",
            Self::HongKongMacaoTaiwanNotifications => "港澳台招生：港澳台最新通知",
            Self::PublicNotices => "信息公开：公示",
            Self::ScoreLines => "信息公开：复试基本分数线",
            Self::AdmissionStatistics => "信息公开：往年报考录取统计",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::MasterGuide => "master-guide",
            Self::MasterNotifications => "master-notifications",
            Self::DoctoralGuide => "doctoral-guide",
            Self::DoctoralNotifications => "doctoral-notifications",
            Self::SummerCampRecommendation => "summer-camp-recommendation",
            Self::HongKongMacaoTaiwanGuide => "hong-kong-macao-taiwan-guide",
            Self::HongKongMacaoTaiwanNotifications => "hong-kong-macao-taiwan-notifications",
            Self::PublicNotices => "public-notices",
            Self::ScoreLines => "score-lines",
            Self::AdmissionStatistics => "admission-statistics",
        }
    }

    fn list_path(self) -> &'static str {
        match self {
            Self::MasterGuide => "47862",
            Self::MasterNotifications => "47863",
            Self::DoctoralGuide => "47864",
            Self::DoctoralNotifications => "47865",
            Self::SummerCampRecommendation => "zxgg",
            Self::HongKongMacaoTaiwanGuide => "47866",
            Self::HongKongMacaoTaiwanNotifications => "47867",
            Self::PublicNotices => "48335",
            Self::ScoreLines => "48336",
            Self::AdmissionStatistics => "48337",
        }
    }

    pub fn list_url(self) -> String {
        format!("{SITE_BASE_URL}{}/list.htm", self.list_path())
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
    pub publish_time: String,
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
        .with_context(|| {
            format!(
                "failed to request graduate admission {} list",
                section.title()
            )
        })?
        .error_for_status()
        .with_context(|| {
            format!(
                "graduate admission {} list returned an error status",
                section.title()
            )
        })?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate admission {} list", section.title()))?;

    parse_article_list(section, page_index, page_url.as_str(), &html)
}

pub async fn list_all_articles(
    client: &common::Client,
    section: ArticleSection,
) -> Result<Vec<Article>> {
    let mut page_index = 1;
    let mut articles = Vec::new();

    loop {
        let page = get_articles(client, section, page_index).await?;
        let fetched = page.articles.len();
        let total = page.total;
        let total_pages = page.total_pages;
        articles.extend(page.articles);

        if fetched == 0
            || total.is_some_and(|total| articles.len() as u64 >= total)
            || total_pages.is_some_and(|total_pages| page_index >= total_pages)
        {
            break;
        }

        page_index += 1;
    }

    Ok(articles)
}

pub async fn read_article(client: &common::Client, url: &str) -> Result<String> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid graduate admission site base URL")?
        .join(url)
        .with_context(|| format!("invalid graduate admission article URL: {url}"))?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request graduate admission article: {url}"))?
        .error_for_status()
        .with_context(|| format!("graduate admission article returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate admission article: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

pub async fn read_column_page(client: &common::Client, section: ArticleSection) -> Result<String> {
    read_page(client, &section.list_url()).await
}

pub async fn read_page(client: &common::Client, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to request graduate admission page: {url}"))?
        .error_for_status()
        .with_context(|| format!("graduate admission page returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate admission page: {url}"))?;

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
        .context("invalid graduate admission site base URL")?
        .join(&path)
        .with_context(|| format!("invalid graduate admission list path: {path}"))
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
            .with_context(|| format!("invalid graduate admission article URL: {href}"))?;
        let title = link
            .value()
            .attr("title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| text_content(link.text()));
        let publish_time = item
            .select(&date_selector)
            .next()
            .map(|date| text_content(date.text()))
            .unwrap_or_default();

        articles.push(Article {
            id: article_id(&url),
            title,
            publish_time,
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
    let title_selector = selector(".arti_title, .col_title h2, title")?;
    let content_selector = selector(".wp_articlecontent, .article, .col_news_con")?;

    let title = document
        .select(&title_selector)
        .next()
        .map(|title| text_content(title.text()))
        .filter(|title| !title.is_empty());
    let content_html = document
        .select(&content_selector)
        .next()
        .map(|content| content.html())
        .unwrap_or_else(|| html.to_string());
    let mut markdown = common::html_to_markdown_with_base_url(&content_html, page_url)?;
    let pdf_urls = embedded_pdf_urls(&document, page_url)?;

    if !pdf_urls.is_empty() {
        markdown.push_str("\n\n## 附件\n\n");
        for url in pdf_urls {
            markdown.push_str(&format!("- [PDF]({url})\n"));
        }
    }

    if let Some(title) = title {
        markdown = format!(
            "# {title}\n\n{}",
            strip_duplicate_heading(&markdown, &title)
        );
    }

    Ok(markdown)
}

fn strip_duplicate_heading<'a>(markdown: &'a str, title: &str) -> &'a str {
    let markdown = markdown.trim_start();
    let Some(rest) = markdown.strip_prefix("# ") else {
        return markdown;
    };
    let Some((heading, body)) = rest.split_once('\n') else {
        return markdown;
    };

    if heading.trim() == title {
        body.trim_start()
    } else {
        markdown
    }
}

fn embedded_pdf_urls(document: &Html, page_url: &str) -> Result<Vec<String>> {
    let base_url = reqwest::Url::parse(page_url)
        .with_context(|| format!("invalid article page URL: {page_url}"))?;
    let pdf_player_selector = selector(".wp_pdf_player, [pdfsrc]")?;
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
                    <span class="news_title"><a href='/c2/4f/c47865a836175/page.htm' title='关于南京大学2026年博士研究生拟录取名单公示'>截断标题</a></span>
                    <span class="news_meta">2026-06-16</span>
                </li>
            </ul>
            <span class="per_page">每页&nbsp;<em class="per_count">14</em>&nbsp;记录&nbsp;</span>
            <span class="all_count">总共&nbsp;<em class="all_count">99</em>&nbsp;记录&nbsp;</span>
            <span class="pages">页码&nbsp;<em class="curr_page">1</em>/<em class="all_pages">8</em></span>
        "#;

        let page = parse_article_list(
            ArticleSection::DoctoralNotifications,
            1,
            "https://yzb.nju.edu.cn/47865/list.htm",
            html,
        )
        .unwrap();

        assert_eq!(page.page_size, Some(14));
        assert_eq!(page.total, Some(99));
        assert_eq!(page.total_pages, Some(8));
        assert_eq!(page.articles.len(), 1);
        assert_eq!(page.articles[0].id, 836175);
        assert_eq!(
            page.articles[0].title,
            "关于南京大学2026年博士研究生拟录取名单公示"
        );
        assert_eq!(page.articles[0].publish_time, "2026-06-16");
        assert_eq!(
            page.articles[0].url,
            "https://yzb.nju.edu.cn/c2/4f/c47865a836175/page.htm"
        );
    }

    #[test]
    fn strips_duplicate_heading() {
        let markdown = strip_duplicate_heading("# 通知\n\n正文", "通知");

        assert_eq!(markdown, "正文");
    }
}
