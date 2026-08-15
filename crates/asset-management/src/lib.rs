use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://zcc.nju.edu.cn/";
const LIST_API_PATH: &str = "njdx/openapi/t/info/list.do";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArticleSection {
    News,
    Notifications,
    NotificationsHousingLand,
    NotificationsHousing,
    NotificationsInfo,
    NotificationsLease,
    NotificationsProjects,
    Regulations,
    RegulationsNational,
    RegulationsHousingLand,
    RegulationsHousing,
    RegulationsInfo,
    RegulationsLease,
    RegulationsProjects,
    DownloadsPolicy,
    DownloadsHousingLand,
    DownloadsHousing,
    DownloadsInfo,
    DownloadsLease,
    DownloadsProjects,
    PenaltyNotices,
    Guides,
    LeaseAnnouncements,
    LeasePublicity,
}

impl ArticleSection {
    pub const ALL: &'static [ArticleSection] = &[
        ArticleSection::News,
        ArticleSection::Notifications,
        ArticleSection::NotificationsHousingLand,
        ArticleSection::NotificationsHousing,
        ArticleSection::NotificationsInfo,
        ArticleSection::NotificationsLease,
        ArticleSection::NotificationsProjects,
        ArticleSection::Regulations,
        ArticleSection::RegulationsNational,
        ArticleSection::RegulationsHousingLand,
        ArticleSection::RegulationsHousing,
        ArticleSection::RegulationsInfo,
        ArticleSection::RegulationsLease,
        ArticleSection::RegulationsProjects,
        ArticleSection::DownloadsPolicy,
        ArticleSection::DownloadsHousingLand,
        ArticleSection::DownloadsHousing,
        ArticleSection::DownloadsInfo,
        ArticleSection::DownloadsLease,
        ArticleSection::DownloadsProjects,
        ArticleSection::PenaltyNotices,
        ArticleSection::Guides,
        ArticleSection::LeaseAnnouncements,
        ArticleSection::LeasePublicity,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::News => "综合新闻",
            Self::Notifications => "通知公告",
            Self::NotificationsHousingLand => "通知公告：公用房与土地管理科",
            Self::NotificationsHousing => "通知公告：住房管理科",
            Self::NotificationsInfo => "通知公告：信息综合科",
            Self::NotificationsLease => "通知公告：出租出借管理科",
            Self::NotificationsProjects => "通知公告：项目管理科",
            Self::Regulations => "规章制度",
            Self::RegulationsNational => "规章制度：国家、省部有关法律法规",
            Self::RegulationsHousingLand => "规章制度：公用房与土地管理科制度",
            Self::RegulationsHousing => "规章制度：住房管理科制度",
            Self::RegulationsInfo => "规章制度：信息综合科制度",
            Self::RegulationsLease => "规章制度：出租出借管理科制度",
            Self::RegulationsProjects => "规章制度：项目管理科制度",
            Self::DownloadsPolicy => "文件下载：政策文件下载",
            Self::DownloadsHousingLand => "文件下载：公用房与土地管理科文件",
            Self::DownloadsHousing => "文件下载：住房管理科文件",
            Self::DownloadsInfo => "文件下载：信息综合科文件",
            Self::DownloadsLease => "文件下载：出租出借管理科文件",
            Self::DownloadsProjects => "文件下载：项目管理科文件",
            Self::PenaltyNotices => "处罚通告",
            Self::Guides => "办事指南",
            Self::LeaseAnnouncements => "公开招租：公告",
            Self::LeasePublicity => "公开招租：公示",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::News => "news",
            Self::Notifications => "notifications",
            Self::NotificationsHousingLand => "notifications-housing-land",
            Self::NotificationsHousing => "notifications-housing",
            Self::NotificationsInfo => "notifications-info",
            Self::NotificationsLease => "notifications-lease",
            Self::NotificationsProjects => "notifications-projects",
            Self::Regulations => "regulations",
            Self::RegulationsNational => "regulations-national",
            Self::RegulationsHousingLand => "regulations-housing-land",
            Self::RegulationsHousing => "regulations-housing",
            Self::RegulationsInfo => "regulations-info",
            Self::RegulationsLease => "regulations-lease",
            Self::RegulationsProjects => "regulations-projects",
            Self::DownloadsPolicy => "downloads-policy",
            Self::DownloadsHousingLand => "downloads-housing-land",
            Self::DownloadsHousing => "downloads-housing",
            Self::DownloadsInfo => "downloads-info",
            Self::DownloadsLease => "downloads-lease",
            Self::DownloadsProjects => "downloads-projects",
            Self::PenaltyNotices => "penalty-notices",
            Self::Guides => "guides",
            Self::LeaseAnnouncements => "lease-announcements",
            Self::LeasePublicity => "lease-publicity",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|section| section.slug() == slug)
    }

    pub fn list_url(self) -> String {
        format!("{SITE_BASE_URL}{}/index.html", self.list_path())
    }

    fn list_path(self) -> &'static str {
        match self {
            Self::News => "bmdt",
            Self::Notifications => "sy/tzzhxx",
            Self::NotificationsHousingLand => "tzgg/gyfytdglk",
            Self::NotificationsHousing => "tzgg/zfglk",
            Self::NotificationsInfo => "tzgg/xxzhk",
            Self::NotificationsLease => "tzgg/czcjk",
            Self::NotificationsProjects => "tzgg/gyzcglk",
            Self::Regulations => "zczd",
            Self::RegulationsNational => "zczd/gjsbygflfg",
            Self::RegulationsHousingLand => "zczd/gyfytdglk/gyfytdglk",
            Self::RegulationsHousing => "zczd/gyfytdglk/zfglk",
            Self::RegulationsInfo => "zczd/gyfytdglk/xxzhk",
            Self::RegulationsLease => "zczd/gyfytdglk/czcjglk",
            Self::RegulationsProjects => "zczd/gyfytdglk/gyzcglk",
            Self::DownloadsPolicy => "wjxz/zcwjxz",
            Self::DownloadsHousingLand => "wjxz/gyfytdglk",
            Self::DownloadsHousing => "wjxz/zfglk",
            Self::DownloadsInfo => "wjxz/xxzhk",
            Self::DownloadsLease => "wjxz/czcjglk",
            Self::DownloadsProjects => "wjxz/bgwjxz",
            Self::PenaltyNotices => "cftg",
            Self::Guides => "sy/bszn",
            Self::LeaseAnnouncements => "gkzz/gg",
            Self::LeasePublicity => "gkzz/gs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticlePage {
    pub section: ArticleSection,
    pub page_index: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub articles: Vec<Article>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub id: u64,
    pub title: String,
    pub publish_date: String,
    pub url: String,
    pub summary: String,
    pub channel_id: u64,
}

#[derive(Debug)]
struct ListPageMeta {
    channel_id: u64,
    page_size: u64,
    total_pages: u64,
    json_page_total: u64,
    data_list: Vec<RawArticlePage>,
}

#[derive(Debug, Deserialize)]
struct RawArticlePage {
    #[serde(default)]
    infolist: Vec<RawArticle>,
}

#[derive(Debug, Deserialize)]
struct RawApiArticlePage {
    #[serde(default)]
    infolist: Vec<RawArticle>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawArticle {
    iid: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    infotitle: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    daytime: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    channelid: u64,
}

#[derive(Debug, Serialize)]
struct EncodedArticleQuery {
    channelid: String,
    pageno: String,
    pagesize: String,
}

pub async fn get_articles(
    client: &common::Client,
    section: ArticleSection,
    page_index: u64,
) -> Result<ArticlePage> {
    if page_index == 0 {
        return Err(anyhow!("page index starts from 1"));
    }

    let response = client
        .get(section.list_url())
        .send()
        .await
        .with_context(|| format!("failed to request asset management {}", section.title()))?
        .error_for_status()
        .with_context(|| {
            format!(
                "asset management {} list returned an error status",
                section.title()
            )
        })?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read asset management {}", section.title()))?;
    let meta = parse_list_page(page_url.as_str(), &html)
        .with_context(|| format!("failed to parse asset management {}", section.title()))?;

    let raw_articles = if meta.total_pages == 0 || page_index > meta.total_pages {
        Vec::new()
    } else if page_index <= meta.json_page_total
        && let Some(page) = meta.data_list.get((page_index - 1) as usize)
    {
        page.infolist.clone()
    } else {
        fetch_api_articles(client, meta.channel_id, page_index, meta.page_size)
            .await?
            .infolist
    };

    Ok(ArticlePage {
        section,
        page_index,
        page_size: meta.page_size,
        total_pages: meta.total_pages,
        articles: raw_articles
            .into_iter()
            .filter_map(raw_article_to_article)
            .collect(),
    })
}

pub async fn list_all_articles(
    client: &common::Client,
    section: ArticleSection,
) -> Result<Vec<Article>> {
    let mut page_index = 1;
    let mut articles = Vec::new();

    loop {
        let page = get_articles(client, section, page_index).await?;
        let total_pages = page.total_pages;
        let fetched = page.articles.len();
        articles.extend(page.articles);

        if fetched == 0 || page_index >= total_pages {
            break;
        }

        page_index += 1;
    }

    Ok(articles)
}

pub async fn read_article(client: &common::Client, url: &str) -> Result<String> {
    let url = normalize_site_url(url)?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request asset management article: {url}"))?
        .error_for_status()
        .with_context(|| format!("asset management article returned an error status: {url}"))?;
    let page_url = normalize_site_url(response.url().as_str())?;
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read asset management article: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

async fn fetch_api_articles(
    client: &common::Client,
    channel_id: u64,
    page_index: u64,
    page_size: u64,
) -> Result<RawApiArticlePage> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid asset management site base URL")?
        .join(LIST_API_PATH)
        .context("invalid asset management list API path")?;

    // 博杉平台前端 post() 会给 API 加 /njdx 前缀，并把表单值 base64 后提交。
    let query = EncodedArticleQuery {
        channelid: encode_form_value(channel_id),
        pageno: encode_form_value(page_index),
        pagesize: encode_form_value(page_size),
    };

    client
        .post(url.clone())
        .form(&query)
        .send()
        .await
        .with_context(|| format!("failed to request asset management list API: {url}"))?
        .error_for_status()
        .with_context(|| format!("asset management list API returned an error status: {url}"))?
        .json()
        .await
        .with_context(|| format!("failed to parse asset management list API response: {url}"))
}

fn parse_list_page(page_url: &str, html: &str) -> Result<ListPageMeta> {
    let channel_id = parse_js_number(html, "var channelId=")?;
    let page_size = parse_js_number(html, "var pageSize=")?;
    let total_pages = parse_js_number(html, "var pageTotal=parseInt(")
        .or_else(|_| parse_js_number(html, "var channelpageTotal=parseInt("))?;
    let json_page_total = parse_js_number(html, "var jsonPageTotal=parseInt(")?;
    let data_list_json = extract_data_list_json(html)?;
    let data_list = serde_json::from_str(data_list_json)
        .with_context(|| format!("failed to parse embedded dataList in {page_url}"))?;

    Ok(ListPageMeta {
        channel_id,
        page_size,
        total_pages,
        json_page_total,
        data_list,
    })
}

fn parse_js_number(html: &str, marker: &str) -> Result<u64> {
    let start = html
        .find(marker)
        .ok_or_else(|| anyhow!("failed to find {marker}"))?
        + marker.len();
    let digits = html[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();

    digits
        .parse()
        .with_context(|| format!("failed to parse number after {marker}"))
}

fn extract_data_list_json(html: &str) -> Result<&str> {
    let marker = "var dataList=";
    let start = html
        .find(marker)
        .ok_or_else(|| anyhow!("failed to find embedded dataList"))?
        + marker.len();
    let rest = &html[start..];
    let end = rest
        .find("var pagesData=")
        .ok_or_else(|| anyhow!("failed to find embedded pagesData after dataList"))?;
    let json = rest[..end].trim().trim_end_matches(';').trim();

    Ok(json)
}

fn raw_article_to_article(raw: RawArticle) -> Option<Article> {
    let url = normalize_site_url(&raw.url).ok()?;
    let title = if raw.title.trim().is_empty() {
        clean_html_text(&raw.infotitle)
    } else {
        clean_html_text(&raw.title)
    };
    if title.is_empty() {
        return None;
    }

    Some(Article {
        id: raw.iid,
        title,
        publish_date: raw.daytime.trim().to_string(),
        url: url.to_string(),
        summary: clean_html_text(&raw.summary),
        channel_id: raw.channelid,
    })
}

fn article_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let body_html = article_body_html(&document)?.unwrap_or_else(|| html.to_string());
    let mut markdown = common::html_to_markdown_with_base_url(&body_html, page_url)?;
    let title = page_title(&document).ok();
    let pdf_urls = embedded_pdf_urls(&document, page_url)?;

    if let Some(title) = title {
        markdown = format!(
            "# {title}\n\n{}",
            strip_duplicate_heading(&markdown, &title)
        );
    }

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

fn article_body_html(document: &Html) -> Result<Option<String>> {
    for css in [".content #word", ".content .words", ".content"] {
        let selector = selector(css)?;
        if let Some(body) = document.select(&selector).next() {
            return Ok(Some(body.html()));
        }
    }

    Ok(None)
}

fn page_title(document: &Html) -> Result<String> {
    let selector = selector(".content .title, title")?;

    document
        .select(&selector)
        .find_map(|element| {
            let title = text_content(element.text());
            (!title.is_empty()).then_some(title)
        })
        .ok_or_else(|| anyhow!("failed to find asset management article title"))
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
        let Ok(url) = normalize_site_url(url.as_str()) else {
            return;
        };
        let url = url.to_string();
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn normalize_site_url(url: &str) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid asset management site base URL")?
        .join(url)
        .with_context(|| format!("invalid asset management URL: {url}"))?;
    if matches!(url.host_str(), Some("zcc.nju.edu.cn")) {
        let _ = url.set_scheme("https");
    }
    url.set_fragment(None);

    Ok(url)
}

fn clean_html_text(text: &str) -> String {
    let fragment = Html::parse_fragment(text);
    let cleaned = text_content(fragment.root_element().text());

    if cleaned.is_empty() {
        text.trim().to_string()
    } else {
        cleaned
    }
}

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<Vec<_>>().join("").trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector).map_err(|error| anyhow!("invalid CSS selector {selector}: {error}"))
}

fn encode_form_value(value: impl ToString) -> String {
    STANDARD.encode(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_article_list() {
        let html = r#"
            <script>
                var channelId=13906;
                var pageSize=15;
                var channelpageTotal=parseInt(1);
                var jsonPageTotal=parseInt(10);
                var dataList=[{"infolist":[{"iid":220707,"title":"公用房","url":"http://zcc.nju.edu.cn/sy/bszn/20220415/i220707.html","daytime":"2022-04-15","summary":"办事指南","channelid":13906}]}];
                var pagesData={"pageTotal":1};
                var pageTotal=parseInt(1);
            </script>
        "#;

        let meta = parse_list_page("https://zcc.nju.edu.cn/sy/bszn/index.html", html).unwrap();
        let article = raw_article_to_article(meta.data_list[0].infolist[0].clone()).unwrap();

        assert_eq!(meta.channel_id, 13906);
        assert_eq!(meta.page_size, 15);
        assert_eq!(meta.total_pages, 1);
        assert_eq!(article.id, 220707);
        assert_eq!(article.title, "公用房");
        assert_eq!(
            article.url,
            "https://zcc.nju.edu.cn/sy/bszn/20220415/i220707.html"
        );
    }

    #[test]
    fn prefers_real_page_total_over_channel_page_window() {
        let html = r#"
            <script>
                var channelId=13929;
                var pageSize=15;
                var channelpageTotal=parseInt(10);
                var jsonPageTotal=parseInt(10);
                var dataList=[];
                var pagesData={"pageTotal":17};
                var pageTotal=parseInt(17);
            </script>
        "#;

        let meta = parse_list_page("https://zcc.nju.edu.cn/gkzz/gg/index.html", html).unwrap();

        assert_eq!(meta.total_pages, 17);
    }

    #[test]
    fn encodes_api_form_values_with_base64() {
        assert_eq!(encode_form_value(13929), "MTM5Mjk=");
        assert_eq!(encode_form_value(11), "MTE=");
        assert_eq!(encode_form_value(15), "MTU=");
    }

    #[test]
    fn strips_duplicate_heading() {
        let markdown = strip_duplicate_heading("# 公用房\n\n正文", "公用房");

        assert_eq!(markdown, "正文");
    }
}
