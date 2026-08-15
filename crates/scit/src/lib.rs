use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://scit.nju.edu.cn/";
const SITE_ID: &str = "327";
const ARTICLES_URL: &str = "https://scit.nju.edu.cn/_wp3services/generalQuery?queryObj=articles";
const ORDERS: &str = r#"[{"field":"top","type":"desc"},{"field":"new","type":"desc"},{"field":"publishTime","type":"desc"}]"#;
const ARTICLE_RETURN_INFOS: &str = r#"[{"field":"title","pattern":[{"name":"lp","value":"999"}],"name":"title"},{"field":"publishTime","pattern":[{"name":"d","value":"yyyy-MM-dd"}],"name":"publishTime"},{"field":"link","name":"link"}]"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArticleSection {
    Notifications,
    ResearchNews,
    PublicInfo,
    Ai,
    ResearchProjects,
    ResearchProjectsNsfc,
    ResearchProjectsMost,
    ResearchProjectsMoe,
    ResearchProjectsJiangsu,
    ResearchProjectsFundamental,
    ResearchProjectsIndustry,
    ResearchProjectsOther,
    Workflow,
    WorkflowProjects,
    WorkflowPlatforms,
    WorkflowAchievements,
    WorkflowContracts,
    WorkflowOther,
    Downloads,
    DownloadsProjects,
    DownloadsPlatforms,
    DownloadsAchievements,
    DownloadsContracts,
    DownloadsOther,
    Institutions,
    InstitutionsLeaders,
    InstitutionsGeneralOffice,
    InstitutionsPlatformOffice,
    InstitutionsVerticalProjectsOffice,
    InstitutionsIndustryOffice,
    InstitutionsAchievementsOffice,
    InstitutionsSpecialOffice,
    InstitutionsSuzhouOffice,
    Policies,
    PoliciesProjects,
    PoliciesPlatforms,
    PoliciesAchievements,
    PoliciesContracts,
    PoliciesOther,
    Platforms,
    PlatformsNational,
    PlatformsMoe,
    PlatformsJiangsu,
    PlatformsOtherMinistry,
    PlatformsUniversity,
    PlatformsIndustry,
    PlatformsNotices,
    Achievements,
    AchievementsAwards,
    AchievementsPapers,
    AchievementsIp,
    AcademicIntegrity,
    AcademicIntegrityPolicies,
    AcademicIntegrityReports,
    AcademicIntegrityStories,
}

impl ArticleSection {
    pub const ALL: &'static [ArticleSection] = &[
        ArticleSection::Notifications,
        ArticleSection::ResearchNews,
        ArticleSection::PublicInfo,
        ArticleSection::Ai,
        ArticleSection::ResearchProjects,
        ArticleSection::ResearchProjectsNsfc,
        ArticleSection::ResearchProjectsMost,
        ArticleSection::ResearchProjectsMoe,
        ArticleSection::ResearchProjectsJiangsu,
        ArticleSection::ResearchProjectsFundamental,
        ArticleSection::ResearchProjectsIndustry,
        ArticleSection::ResearchProjectsOther,
        ArticleSection::Workflow,
        ArticleSection::WorkflowProjects,
        ArticleSection::WorkflowPlatforms,
        ArticleSection::WorkflowAchievements,
        ArticleSection::WorkflowContracts,
        ArticleSection::WorkflowOther,
        ArticleSection::Downloads,
        ArticleSection::DownloadsProjects,
        ArticleSection::DownloadsPlatforms,
        ArticleSection::DownloadsAchievements,
        ArticleSection::DownloadsContracts,
        ArticleSection::DownloadsOther,
        ArticleSection::Institutions,
        ArticleSection::InstitutionsLeaders,
        ArticleSection::InstitutionsGeneralOffice,
        ArticleSection::InstitutionsPlatformOffice,
        ArticleSection::InstitutionsVerticalProjectsOffice,
        ArticleSection::InstitutionsIndustryOffice,
        ArticleSection::InstitutionsAchievementsOffice,
        ArticleSection::InstitutionsSpecialOffice,
        ArticleSection::InstitutionsSuzhouOffice,
        ArticleSection::Policies,
        ArticleSection::PoliciesProjects,
        ArticleSection::PoliciesPlatforms,
        ArticleSection::PoliciesAchievements,
        ArticleSection::PoliciesContracts,
        ArticleSection::PoliciesOther,
        ArticleSection::Platforms,
        ArticleSection::PlatformsNational,
        ArticleSection::PlatformsMoe,
        ArticleSection::PlatformsJiangsu,
        ArticleSection::PlatformsOtherMinistry,
        ArticleSection::PlatformsUniversity,
        ArticleSection::PlatformsIndustry,
        ArticleSection::PlatformsNotices,
        ArticleSection::Achievements,
        ArticleSection::AchievementsAwards,
        ArticleSection::AchievementsPapers,
        ArticleSection::AchievementsIp,
        ArticleSection::AcademicIntegrity,
        ArticleSection::AcademicIntegrityPolicies,
        ArticleSection::AcademicIntegrityReports,
        ArticleSection::AcademicIntegrityStories,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Notifications => "通知公告",
            Self::ResearchNews => "科研动态",
            Self::PublicInfo => "公示信息",
            Self::Ai => "AI4S专栏",
            Self::ResearchProjects => "科研项目",
            Self::ResearchProjectsNsfc => "国家自然科学基金",
            Self::ResearchProjectsMost => "科技部项目",
            Self::ResearchProjectsMoe => "教育部项目",
            Self::ResearchProjectsJiangsu => "江苏省项目",
            Self::ResearchProjectsFundamental => "中央高校基本科研业务费",
            Self::ResearchProjectsIndustry => "横向科研项目",
            Self::ResearchProjectsOther => "其他项目",
            Self::Workflow => "工作流程",
            Self::WorkflowProjects => "工作流程：科研项目",
            Self::WorkflowPlatforms => "工作流程：科研平台",
            Self::WorkflowAchievements => "工作流程：科技成果",
            Self::WorkflowContracts => "工作流程：科技合同",
            Self::WorkflowOther => "工作流程：其他",
            Self::Downloads => "相关下载",
            Self::DownloadsProjects => "相关下载：科研项目",
            Self::DownloadsPlatforms => "相关下载：科研平台",
            Self::DownloadsAchievements => "相关下载：科技成果",
            Self::DownloadsContracts => "相关下载：科技合同",
            Self::DownloadsOther => "相关下载：其他",
            Self::Institutions => "机构设置",
            Self::InstitutionsLeaders => "院领导",
            Self::InstitutionsGeneralOffice => "综合办公室",
            Self::InstitutionsPlatformOffice => "科研平台办公室",
            Self::InstitutionsVerticalProjectsOffice => "纵向项目办公室",
            Self::InstitutionsIndustryOffice => "产业合作办公室",
            Self::InstitutionsAchievementsOffice => "成果奖励办公室",
            Self::InstitutionsSpecialOffice => "专项科研办公室",
            Self::InstitutionsSuzhouOffice => "苏州校区办公室",
            Self::Policies => "政策法规",
            Self::PoliciesProjects => "政策法规：科研项目",
            Self::PoliciesPlatforms => "政策法规：科研平台",
            Self::PoliciesAchievements => "政策法规：科技成果",
            Self::PoliciesContracts => "政策法规：科技合同",
            Self::PoliciesOther => "政策法规：其他",
            Self::Platforms => "科研平台",
            Self::PlatformsNational => "国家级科研平台",
            Self::PlatformsMoe => "教育部科研平台",
            Self::PlatformsJiangsu => "江苏省科研平台",
            Self::PlatformsOtherMinistry => "其他部委办局级科研平台",
            Self::PlatformsUniversity => "校级科研机构",
            Self::PlatformsIndustry => "校企产学研合作机构",
            Self::PlatformsNotices => "科研平台工作通知",
            Self::Achievements => "科技成果",
            Self::AchievementsAwards => "科技奖励",
            Self::AchievementsPapers => "科技论文",
            Self::AchievementsIp => "知识产权",
            Self::AcademicIntegrity => "学风建设",
            Self::AcademicIntegrityPolicies => "政策文件",
            Self::AcademicIntegrityReports => "年度报告",
            Self::AcademicIntegrityStories => "先进事迹",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Notifications => "notifications",
            Self::ResearchNews => "research-news",
            Self::PublicInfo => "public-info",
            Self::Ai => "ai",
            Self::ResearchProjects => "research-projects",
            Self::ResearchProjectsNsfc => "research-projects-nsfc",
            Self::ResearchProjectsMost => "research-projects-most",
            Self::ResearchProjectsMoe => "research-projects-moe",
            Self::ResearchProjectsJiangsu => "research-projects-jiangsu",
            Self::ResearchProjectsFundamental => "research-projects-fundamental",
            Self::ResearchProjectsIndustry => "research-projects-industry",
            Self::ResearchProjectsOther => "research-projects-other",
            Self::Workflow => "workflow",
            Self::WorkflowProjects => "workflow-projects",
            Self::WorkflowPlatforms => "workflow-platforms",
            Self::WorkflowAchievements => "workflow-achievements",
            Self::WorkflowContracts => "workflow-contracts",
            Self::WorkflowOther => "workflow-other",
            Self::Downloads => "downloads",
            Self::DownloadsProjects => "downloads-projects",
            Self::DownloadsPlatforms => "downloads-platforms",
            Self::DownloadsAchievements => "downloads-achievements",
            Self::DownloadsContracts => "downloads-contracts",
            Self::DownloadsOther => "downloads-other",
            Self::Institutions => "institutions",
            Self::InstitutionsLeaders => "institutions-leaders",
            Self::InstitutionsGeneralOffice => "institutions-general-office",
            Self::InstitutionsPlatformOffice => "institutions-platform-office",
            Self::InstitutionsVerticalProjectsOffice => "institutions-vertical-projects-office",
            Self::InstitutionsIndustryOffice => "institutions-industry-office",
            Self::InstitutionsAchievementsOffice => "institutions-achievements-office",
            Self::InstitutionsSpecialOffice => "institutions-special-office",
            Self::InstitutionsSuzhouOffice => "institutions-suzhou-office",
            Self::Policies => "policies",
            Self::PoliciesProjects => "policies-projects",
            Self::PoliciesPlatforms => "policies-platforms",
            Self::PoliciesAchievements => "policies-achievements",
            Self::PoliciesContracts => "policies-contracts",
            Self::PoliciesOther => "policies-other",
            Self::Platforms => "platforms",
            Self::PlatformsNational => "platforms-national",
            Self::PlatformsMoe => "platforms-moe",
            Self::PlatformsJiangsu => "platforms-jiangsu",
            Self::PlatformsOtherMinistry => "platforms-other-ministry",
            Self::PlatformsUniversity => "platforms-university",
            Self::PlatformsIndustry => "platforms-industry",
            Self::PlatformsNotices => "platforms-notices",
            Self::Achievements => "achievements",
            Self::AchievementsAwards => "achievements-awards",
            Self::AchievementsPapers => "achievements-papers",
            Self::AchievementsIp => "achievements-ip",
            Self::AcademicIntegrity => "academic-integrity",
            Self::AcademicIntegrityPolicies => "academic-integrity-policies",
            Self::AcademicIntegrityReports => "academic-integrity-reports",
            Self::AcademicIntegrityStories => "academic-integrity-stories",
        }
    }

    pub fn list_path(self) -> &'static str {
        match self {
            Self::Notifications => "10916",
            Self::ResearchNews => "11003",
            Self::PublicInfo => "cs_39328",
            Self::Ai => "AI",
            Self::ResearchProjects => "10921",
            Self::ResearchProjectsNsfc => "10946",
            Self::ResearchProjectsMost => "10945",
            Self::ResearchProjectsMoe => "10947",
            Self::ResearchProjectsJiangsu => "10949",
            Self::ResearchProjectsFundamental => "10948",
            Self::ResearchProjectsIndustry => "10950",
            Self::ResearchProjectsOther => "10951",
            Self::Workflow => "10918",
            Self::WorkflowProjects => "10952",
            Self::WorkflowPlatforms => "10956",
            Self::WorkflowAchievements => "10953",
            Self::WorkflowContracts => "10958",
            Self::WorkflowOther => "10961",
            Self::Downloads => "10924",
            Self::DownloadsProjects => "10962",
            Self::DownloadsPlatforms => "10966",
            Self::DownloadsAchievements => "10963",
            Self::DownloadsContracts => "10968",
            Self::DownloadsOther => "10971",
            Self::Institutions => "10915",
            Self::InstitutionsLeaders => "11179",
            Self::InstitutionsGeneralOffice => "11180",
            Self::InstitutionsPlatformOffice => "11182",
            Self::InstitutionsVerticalProjectsOffice => "11181",
            Self::InstitutionsIndustryOffice => "11184",
            Self::InstitutionsAchievementsOffice => "11183",
            Self::InstitutionsSpecialOffice => "11185",
            Self::InstitutionsSuzhouOffice => "11186",
            Self::Policies => "10917",
            Self::PoliciesProjects => "10935",
            Self::PoliciesPlatforms => "10939",
            Self::PoliciesAchievements => "10936",
            Self::PoliciesContracts => "10941",
            Self::PoliciesOther => "10944",
            Self::Platforms => "10922",
            Self::PlatformsNational => "gjzdsys",
            Self::PlatformsMoe => "jybzdsys",
            Self::PlatformsJiangsu => "jsszdsys",
            Self::PlatformsOtherMinistry => "gczx",
            Self::PlatformsUniversity => "qtpt",
            Self::PlatformsIndustry => "xqcxyhzjg",
            Self::PlatformsNotices => "xtcxzx",
            Self::Achievements => "10923",
            Self::AchievementsAwards => "10972",
            Self::AchievementsPapers => "10973",
            Self::AchievementsIp => "10974",
            Self::AcademicIntegrity => "10925",
            Self::AcademicIntegrityPolicies => "10982",
            Self::AcademicIntegrityReports => "10984",
            Self::AcademicIntegrityStories => "10985",
        }
    }

    pub fn list_url(self) -> String {
        format!("{SITE_BASE_URL}{}/list.htm", self.list_path())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticlePage {
    pub status: i32,
    pub result: String,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub data: Vec<Article>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub publish_time: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub wap_url: Option<String>,
    #[serde(default)]
    pub true_wap_url: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub publisher_id: Option<u64>,
    #[serde(default)]
    pub visit_count: Option<u64>,
    #[serde(default)]
    pub mirc_img_path: Option<String>,
    #[serde(default)]
    pub site_art_id: Option<u64>,
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

pub async fn get_articles(
    client: &common::Client,
    section: ArticleSection,
    page_index: u64,
    page_size: u64,
) -> Result<ArticlePage> {
    let column_id = resolve_column_id(client, section).await?;
    let mut page = get_articles_by_column_id(client, &column_id, page_index.max(1), page_size)
        .await
        .with_context(|| format!("failed to request {}", section.title()))?;

    for article in &mut page.data {
        if let Ok(url) = normalize_scit_url(&article.url) {
            article.url = url.to_string();
        }
    }

    Ok(page)
}

pub async fn list_all_articles(
    client: &common::Client,
    section: ArticleSection,
    page_size: u64,
) -> Result<Vec<Article>> {
    let page_size = page_size.max(1);
    let mut page_index = 1;
    let mut articles = Vec::new();

    loop {
        let page = get_articles(client, section, page_index, page_size).await?;
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

pub async fn read_article(client: &common::Client, url: &str) -> Result<String> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid SCIT site base URL")?
        .join(url)
        .with_context(|| format!("invalid SCIT article URL: {url}"))?;
    let url = normalize_scit_url(url.as_str())?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request SCIT article: {url}"))?
        .error_for_status()
        .with_context(|| format!("SCIT article returned an error status: {url}"))?;
    let page_url = normalize_scit_url(response.url().as_str())?;
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read SCIT article body: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

async fn resolve_column_id(client: &common::Client, section: ArticleSection) -> Result<String> {
    if section
        .list_path()
        .bytes()
        .all(|byte| byte.is_ascii_digit())
    {
        return Ok(section.list_path().to_string());
    }

    let response = client
        .get(section.list_url())
        .send()
        .await
        .with_context(|| format!("failed to request {} list page", section.title()))?
        .error_for_status()
        .with_context(|| format!("{} list page returned an error status", section.title()))?;
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read {} list page body", section.title()))?;
    let document = Html::parse_document(&html);

    visit_count_column_id(&document)
        .map(|id| id.to_string())
        .ok_or_else(|| anyhow!("failed to resolve column id for {}", section.title()))
}

async fn get_articles_by_column_id(
    client: &common::Client,
    column_id: &str,
    page_index: u64,
    page_size: u64,
) -> Result<ArticlePage> {
    Ok(client
        .post(ARTICLES_URL)
        .form(&ArticleQuery {
            site_id: SITE_ID,
            column_id,
            page_index,
            rows: page_size.max(1),
            orders: ORDERS,
            return_infos: ARTICLE_RETURN_INFOS,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

fn article_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let content_selector = selector(".wp_articlecontent, .article, .col_news_con")?;
    let content_html = document
        .select(&content_selector)
        .next()
        .map(|content| content.html())
        .unwrap_or_else(|| html.to_string());
    let mut markdown = common::html_to_markdown_with_base_url(&content_html, page_url)?;
    let title = page_title(&document).ok();
    let pdf_urls = extract_pdf_urls(html, page_url)?;

    if let Some(title) = title {
        let trimmed = markdown.trim_start();
        let expected_heading = format!("# {title}");
        if !trimmed.starts_with(&expected_heading) {
            markdown = format!("{expected_heading}\n\n{trimmed}");
        }
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

fn page_title(document: &Html) -> Result<String> {
    let selector = selector(".arti_title, .col_title h2, title")?;

    document
        .select(&selector)
        .find_map(|element| {
            let title = text_content(element.text());
            (!title.is_empty()).then_some(title)
        })
        .ok_or_else(|| anyhow!("failed to find SCIT article title"))
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
        let Ok(url) = normalize_scit_url(url.as_str()) else {
            return;
        };
        let url = url.to_string();
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn visit_count_column_id(document: &Html) -> Option<u64> {
    let selector = selector(r#"img[src*="_visitcount"]"#).ok()?;

    for element in document.select(&selector) {
        let src = element.value().attr("src")?;
        let url = reqwest::Url::parse(SITE_BASE_URL).ok()?.join(src).ok()?;
        for (name, value) in url.query_pairs() {
            if name == "columnId" {
                if let Ok(id) = value.parse() {
                    return Some(id);
                }
            }
        }
    }

    None
}

fn normalize_scit_url(url: &str) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(url).with_context(|| format!("invalid SCIT URL: {url}"))?;
    let _ = url.set_scheme("https");
    if matches!(url.host_str(), Some("scit.nju.edu.cn")) {
        let _ = url.set_host(Some("scit.nju.edu.cn"));
    }
    url.set_fragment(None);

    Ok(url)
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
    fn parses_column_id_from_visit_count() {
        let html = r#"<img src="/_visitcount?siteId=327&type=2&columnId=11172" />"#;
        let document = Html::parse_document(html);

        assert_eq!(visit_count_column_id(&document), Some(11172));
    }

    #[test]
    fn exposes_required_sections() {
        assert_eq!(ArticleSection::ResearchNews.list_path(), "11003");
        assert_eq!(ArticleSection::PublicInfo.list_path(), "cs_39328");
        assert_eq!(ArticleSection::Ai.list_path(), "AI");
        assert_eq!(ArticleSection::ResearchProjectsNsfc.list_path(), "10946");
        assert_eq!(ArticleSection::PlatformsNational.list_path(), "gjzdsys");
    }
}
