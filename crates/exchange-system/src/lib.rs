use anyhow::{Context, Result};
use serde::Deserialize;

const BASE_URL: &str = "http://elite.nju.edu.cn/exchangesystem/index/";
const NOTICE_LIST_URL: &str = "http://elite.nju.edu.cn/exchangesystem/index/moreList";
const PROJECT_LIST_URL: &str = "http://elite.nju.edu.cn/exchangesystem/index/moreXmList";
const ME: &str = "c3lzLmluZGV4";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub code: i32,
    pub msg: Option<String>,
    pub page_index: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub count: u64,
    pub data: Vec<T>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Notice {
    pub pid: u64,
    /// 标题
    pub bt: String,
    /// 正文 HTML
    pub nr: String,
    #[serde(rename = "createDate")]
    pub create_date: Option<String>,
    #[serde(rename = "createBy")]
    pub create_by: Option<String>,
    /// 附件路径
    pub fj: Option<String>,
    /// 附件名称
    pub fjmc: Option<String>,
    /// 是否置顶
    pub sfzd: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Project {
    pub pid: u64,
    /// 项目名称
    pub mc: String,
    /// 交流时间
    pub jlsj: Option<String>,
    /// 报名截止时间
    pub bmjzsj: Option<String>,
    /// 申请要求链接
    pub sqyq: Option<String>,
    /// 项目描述 HTML
    pub xmms: Option<String>,
    pub bz: Option<String>,
    /// 附件路径
    pub fj: Option<String>,
    /// 附件名称
    pub fjmc: Option<String>,
    /// 是否置顶
    pub sfzd: Option<bool>,
}

/// 获取交换生系统新闻通知列表。
pub async fn get_notices(
    client: &common::Client,
    page_index: u64,
    page_size: u64,
) -> Result<Page<Notice>> {
    get_page(client, NOTICE_LIST_URL, page_index, page_size).await
}

/// 获取交换生系统项目列表。
pub async fn get_projects(
    client: &common::Client,
    page_index: u64,
    page_size: u64,
) -> Result<Page<Project>> {
    get_page(client, PROJECT_LIST_URL, page_index, page_size).await
}

async fn get_page<T: serde::de::DeserializeOwned>(
    client: &common::Client,
    url: &str,
    page_index: u64,
    page_size: u64,
) -> Result<Page<T>> {
    client
        .get(url)
        .query(&[("page", page_index), ("limit", page_size)])
        .query(&[(".me", ME)])
        .send()
        .await
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("GET {url} returned an error status"))?
        .json()
        .await
        .with_context(|| format!("failed to decode GET {url} response"))
}

/// 将通知正文 HTML 转为 Markdown，并补全相对链接。
pub fn notice_to_markdown(notice: &Notice) -> Result<String> {
    html_fragment_to_markdown(&notice.nr)
        .with_context(|| format!("failed to convert notice {} to markdown", notice.pid))
}

/// 将项目描述 HTML 转为 Markdown，并补全相对链接。
pub fn project_to_markdown(project: &Project) -> Result<String> {
    let mut html = String::new();

    if let Some(description) = &project.xmms {
        html.push_str(description);
    }
    if let Some(requirement_url) = &project.sqyq {
        html.push_str("<p>申请要求：<a href=\"");
        html.push_str(requirement_url);
        html.push_str("\">查看申请要求</a></p>");
    }
    if let Some(remark) = &project.bz {
        html.push_str("<h2>备注</h2>");
        html.push_str(remark);
    }

    html_fragment_to_markdown(&html)
        .with_context(|| format!("failed to convert project {} to markdown", project.pid))
}

fn html_fragment_to_markdown(html: &str) -> Result<String> {
    common::html_to_markdown_with_base_url(html, BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_notices() {
        let client = common::ClientBuilder::new(reqwest::Client::new()).build();
        let notices = get_notices(&client, 1, 20).await.unwrap();

        assert!(!notices.data.is_empty());
    }
}
