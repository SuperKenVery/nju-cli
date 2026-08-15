use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{SITE_BASE_URL, SITE_ID};

const ARTICLES_URL: &str = "https://jw.nju.edu.cn/_wp3services/generalQuery?queryObj=articles";
const INSTITUTIONS_URL: &str = "https://jw.nju.edu.cn/65048/list.htm";
const ANNOUNCEMENTS_COLUMN_ID: &str = "26263";
const ORDERS: &str = r#"[{"field":"top","type":"desc"},{"field":"new","type":"desc"},{"field":"publishTime","type":"desc"}]"#;
// lp是限制标题字数，多出来的变省略号
const ANNOUNCEMENT_RETURN_INFOS: &str = r#"[{"field":"title","pattern":[{"name":"lp","value":"999"}],"name":"title"},{"field":"f1","name":"f1"},{"field":"publishTime","pattern":[{"name":"d","value":"MM-dd"}],"name":"publishTime"},{"field":"topImg","name":"topImg"},{"field":"newImg","name":"newImg"},{"field":"link","name":"link"}]"#;
const ARTICLE_RETURN_INFOS: &str = r#"[{"field":"title","pattern":[{"name":"lp","value":"999"}],"name":"title"},{"field":"publishTime","pattern":[{"name":"d","value":"yyyy-MM-dd"}],"name":"publishTime"},{"field":"link","name":"link"}]"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArticleColumn {
    MinistryDocuments,
    SchoolDocuments,
    StudentHandbook,
    TeacherHandbook,
    Procedures,
    DepartmentLeaders,
    Institutions,
    CalendarCatalog,
    Forms,
    Templates,
}

impl ArticleColumn {
    pub fn column_id(self) -> &'static str {
        match self {
            Self::MinistryDocuments => "24747",
            Self::StudentHandbook => "24748",
            Self::TeacherHandbook => "24749",
            Self::SchoolDocuments => "24750",
            Self::Procedures => "24751",
            Self::DepartmentLeaders => "65047",
            Self::Institutions => "65048",
            Self::CalendarCatalog => "24809",
            Self::Forms => "24815",
            Self::Templates => "24816",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::MinistryDocuments => "教育部文件",
            Self::SchoolDocuments => "学校文件",
            Self::StudentHandbook => "学生手册",
            Self::TeacherHandbook => "教师手册",
            Self::Procedures => "办事流程",
            Self::DepartmentLeaders => "部门领导",
            Self::Institutions => "机构设置",
            Self::CalendarCatalog => "校历目录",
            Self::Forms => "各类表格",
            Self::Templates => "各类模板",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticlePage {
    pub status: i32,
    pub result: String,
    pub total: u64,
    pub data: Vec<Article>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: u64,
    pub title: String,
    /// 通知标签，比如"信息,毕业"
    #[serde(rename = "f1")]
    pub tags: Option<String>,
    pub publish_time: String,
    /// 置顶标的图标URL
    pub top_img: Option<String>,
    pub new_img: Option<String>,
    pub wap_url: Option<String>,
    pub true_wap_url: Option<String>,
    pub url: String,
    pub publisher: Option<String>,
    pub publisher_id: Option<u64>,
    pub visit_count: Option<u64>,
    pub mirc_img_path: Option<String>,
    pub site_art_id: Option<u64>,
}

pub type AnnouncementPage = ArticlePage;
pub type Announcement = Article;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Institution {
    pub column_id: u64,
    pub title: String,
    pub list_url: String,
    pub article: Article,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArticleQuery<'a> {
    site_id: &'a str,
    column_id: &'a str,
    page_index: u64,
    rows: u64,
    orders: &'a str,
    return_infos: &'a str,
}

/// 获取教务网「公告通知」列表。
///
/// `page_size` 对应请求中的 `rows`，即一页返回的公告数量。该接口不需要额外
/// header 或 cookie；调用方传入的 `client` 可复用已有 reqwest session。
pub async fn get_announcements(
    client: &common::Client,
    page_index: u64,
    page_size: u64,
) -> Result<AnnouncementPage> {
    get_articles_by_column_id(
        client,
        ANNOUNCEMENTS_COLUMN_ID,
        page_index,
        page_size,
        ANNOUNCEMENT_RETURN_INFOS,
    )
    .await
}

/// 获取教务网指定栏目的一页文章列表。
pub async fn get_column_articles(
    client: &common::Client,
    column: ArticleColumn,
    page_index: u64,
    page_size: u64,
) -> Result<ArticlePage> {
    get_column_articles_by_id(client, column.column_id(), page_index, page_size).await
}

/// 获取教务网指定栏目 ID 的一页文章列表。
pub async fn get_column_articles_by_id(
    client: &common::Client,
    column_id: &str,
    page_index: u64,
    page_size: u64,
) -> Result<ArticlePage> {
    get_articles_by_column_id(
        client,
        column_id,
        page_index,
        page_size,
        ARTICLE_RETURN_INFOS,
    )
    .await
}

/// 获取教务网「机构设置」下属机构列表。
pub async fn get_institutions(client: &common::Client) -> Result<Vec<Institution>> {
    let response = client
        .get(INSTITUTIONS_URL)
        .send()
        .await
        .context("failed to request institutions page")?
        .error_for_status()
        .context("institutions page returned an error status")?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .context("failed to read institutions page")?;

    parse_institution_columns(client, &html, page_url.as_str()).await
}

/// 获取教务网指定栏目下所有文章。
pub async fn list_all_column_articles(
    client: &common::Client,
    column: ArticleColumn,
    page_size: u64,
) -> Result<Vec<Article>> {
    let page_size = page_size.max(1);
    let mut page_index = 1;
    let mut articles = Vec::new();

    loop {
        let page = get_column_articles(client, column, page_index, page_size).await?;
        let total = page.total;
        let fetched = page.data.len();
        articles.extend(page.data);

        if articles.len() as u64 >= total || fetched == 0 {
            break;
        }

        page_index += 1;
    }

    Ok(articles)
}

async fn parse_institution_columns(
    client: &common::Client,
    html: &str,
    page_url: &str,
) -> Result<Vec<Institution>> {
    let document = Html::parse_document(html);
    let base_url = reqwest::Url::parse(page_url)
        .with_context(|| format!("invalid institutions page URL: {page_url}"))?;
    let selector = selector(".wp_column.selected .sub-item-link")?;
    let mut institutions = Vec::new();

    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let Some(column_id) = institution_column_id(href) else {
            continue;
        };
        let Some(list_url) = base_url.join(href).ok() else {
            continue;
        };
        let title = text_content(element.text());
        if title.is_empty() {
            continue;
        }

        let article = get_column_articles_by_id(client, &column_id.to_string(), 1, 1)
            .await
            .with_context(|| format!("failed to list institution column {column_id}"))?
            .data
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no article found in institution column {column_id}"))?;

        institutions.push(Institution {
            column_id,
            title,
            list_url: list_url.to_string(),
            article,
        });
    }

    Ok(institutions)
}

fn institution_column_id(href: &str) -> Option<u64> {
    let first_segment = href
        .trim_start_matches('/')
        .split('/')
        .next()
        .filter(|segment| !segment.is_empty())?;
    let column_id = first_segment.parse::<u64>().ok()?;

    // 65048 是「机构设置」总览页；下属机构目前都在 65051-65060。
    (column_id != 65048).then_some(column_id)
}

async fn get_articles_by_column_id(
    client: &common::Client,
    column_id: &str,
    page_index: u64,
    page_size: u64,
    return_infos: &str,
) -> Result<ArticlePage> {
    Ok(client
        .post(ARTICLES_URL)
        .form(&ArticleQuery {
            site_id: SITE_ID,
            column_id,
            page_index,
            rows: page_size,
            orders: ORDERS,
            return_infos,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// 读取公告页面，并转换为 Markdown。
pub async fn read_announcement(client: &common::Client, url: &str) -> Result<String> {
    read_article(client, url).await
}

/// 读取教务网页面，并转换为 Markdown。
///
/// `url` 可以是栏目列表返回的相对链接或完整链接。Markdown 中的相对链接会基于
/// 最终页面地址补全为绝对链接。
pub async fn read_article(client: &common::Client, url: &str) -> Result<String> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid academic affairs site base URL")?
        .join(url)
        .with_context(|| format!("invalid academic affairs article URL: {url}"))?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request academic affairs article: {url}"))?
        .error_for_status()
        .with_context(|| format!("academic affairs article returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read academic affairs article: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

fn article_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let title_selector = selector(".arti_title, .col_title h2, title")?;
    let content_selector = selector(".wp_articlecontent, .read, .col_news_con")?;

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
    let attachment_urls = embedded_pdf_urls(&document, page_url)?;

    if !attachment_urls.is_empty() {
        markdown.push_str("\n\n## 附件\n\n");
        for url in attachment_urls {
            markdown.push_str(&format!("- [PDF]({url})\n"));
        }
    }

    if let Some(title) = title {
        markdown = format!("# {title}\n\n{}", markdown.trim_start());
    }

    Ok(markdown)
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

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<String>().trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector)
        .map_err(|error| anyhow::anyhow!("invalid CSS selector {selector}: {error}"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converts_embedded_pdf_to_attachment_link() {
        let html = r#"
            <h1 class="arti_title">教育部文件</h1>
            <div class="wp_articlecontent">
                <p><div class="wp_pdf_player" pdfsrc="/_upload/article/files/a/file.pdf"></div></p>
            </div>
        "#;

        let markdown =
            article_html_to_markdown(html, "https://jw.nju.edu.cn/a/b/page.htm").unwrap();

        assert!(markdown.contains("# 教育部文件"));
        assert!(markdown.contains("## 附件"));
        assert!(markdown.contains("https://jw.nju.edu.cn/_upload/article/files/a/file.pdf"));
    }
}
