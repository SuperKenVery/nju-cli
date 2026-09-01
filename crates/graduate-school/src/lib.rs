use anyhow::{Context, Result, anyhow};
use reqwest::{Url, header::CONTENT_TYPE};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

const SITE_BASE_URL: &str = "https://grawww.nju.edu.cn/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArticleSection {
    Notifications,
    ServiceGuides,
    TrainingRegulations,
    NationalSponsoredPrograms,
    UniversityExchangePrograms,
    TrainingInnovationProjects,
    CombinedMasterDoctor,
    CourseDevelopment,
    TrainingStudentStatus,
    TrainingDownloads,
    DegreeWork,
    SupervisorWork,
    ExcellentTheses,
    DegreeImprovementProgram,
    DegreeDownloads,
    HonoraryDoctors,
    StudentPolicies,
    GraduationCertificates,
    StudentDownloads,
    StudentRegistration,
}

impl ArticleSection {
    pub const ALL: &'static [ArticleSection] = &[
        Self::Notifications,
        Self::ServiceGuides,
        Self::TrainingRegulations,
        Self::NationalSponsoredPrograms,
        Self::UniversityExchangePrograms,
        Self::TrainingInnovationProjects,
        Self::CombinedMasterDoctor,
        Self::CourseDevelopment,
        Self::TrainingStudentStatus,
        Self::TrainingDownloads,
        Self::DegreeWork,
        Self::SupervisorWork,
        Self::ExcellentTheses,
        Self::DegreeImprovementProgram,
        Self::DegreeDownloads,
        Self::HonoraryDoctors,
        Self::StudentPolicies,
        Self::GraduationCertificates,
        Self::StudentDownloads,
        Self::StudentRegistration,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Notifications => "通知公告",
            Self::ServiceGuides => "办事指南",
            Self::TrainingRegulations => "培养工作：相关规定",
            Self::NationalSponsoredPrograms => "培养工作：国际交流：高水平公派项目",
            Self::UniversityExchangePrograms => "培养工作：国际交流：校级交流项目",
            Self::TrainingInnovationProjects => "培养工作：各类项目：创新工程",
            Self::CombinedMasterDoctor => "培养工作：各类项目：硕博连读",
            Self::CourseDevelopment => "培养工作：各类项目：课程建设",
            Self::TrainingStudentStatus => "培养工作：学籍相关",
            Self::TrainingDownloads => "培养工作：下载专区",
            Self::DegreeWork => "学位与导师：学位工作",
            Self::SupervisorWork => "学位与导师：导师工作",
            Self::ExcellentTheses => "学位与导师：项目管理：优秀学位论文",
            Self::DegreeImprovementProgram => "学位与导师：项目管理：提升计划",
            Self::DegreeDownloads => "学位与导师：下载专区",
            Self::HonoraryDoctors => "学位与导师：名誉博士",
            Self::StudentPolicies => "学籍与奖助管理：相关政策规定",
            Self::GraduationCertificates => "学籍与奖助管理：学籍学历管理：毕业证书",
            Self::StudentDownloads => "学籍与奖助管理：学籍学历管理：下载专区",
            Self::StudentRegistration => "学籍与奖助管理：学籍学历管理：学籍注册",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Notifications => "notifications",
            Self::ServiceGuides => "service-guides",
            Self::TrainingRegulations => "training-regulations",
            Self::NationalSponsoredPrograms => "national-sponsored-programs",
            Self::UniversityExchangePrograms => "university-exchange-programs",
            Self::TrainingInnovationProjects => "training-innovation-projects",
            Self::CombinedMasterDoctor => "combined-master-doctor",
            Self::CourseDevelopment => "course-development",
            Self::TrainingStudentStatus => "training-student-status",
            Self::TrainingDownloads => "training-downloads",
            Self::DegreeWork => "degree-work",
            Self::SupervisorWork => "supervisor-work",
            Self::ExcellentTheses => "excellent-theses",
            Self::DegreeImprovementProgram => "degree-improvement-program",
            Self::DegreeDownloads => "degree-downloads",
            Self::HonoraryDoctors => "honorary-doctors",
            Self::StudentPolicies => "student-policies",
            Self::GraduationCertificates => "graduation-certificates",
            Self::StudentDownloads => "student-downloads",
            Self::StudentRegistration => "student-registration",
        }
    }

    fn list_path(self) -> &'static str {
        match self {
            Self::Notifications => "905",
            Self::ServiceGuides => "906",
            Self::TrainingRegulations => "55625",
            Self::NationalSponsoredPrograms => "55641",
            Self::UniversityExchangePrograms => "55642",
            Self::TrainingInnovationProjects => "55643",
            Self::CombinedMasterDoctor => "55644",
            Self::CourseDevelopment => "55645",
            Self::TrainingStudentStatus => "xjxg",
            Self::TrainingDownloads => "55628",
            Self::DegreeWork => "55018",
            Self::SupervisorWork => "55019",
            Self::ExcellentTheses => "55020",
            Self::DegreeImprovementProgram => "55017",
            Self::DegreeDownloads => "xzzq",
            Self::HonoraryDoctors => "mybs",
            Self::StudentPolicies => "xgzcgd",
            Self::GraduationCertificates => "byzs",
            Self::StudentDownloads => "xzzq_62790",
            Self::StudentRegistration => "xjzc",
        }
    }

    pub fn list_url(self) -> String {
        format!("{SITE_BASE_URL}{}/list.htm", self.list_path())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InfoPage {
    GraduateSchoolIntroduction,
    DepartmentLeaders,
    GeneralOffice,
    AdmissionsOffice,
    TrainingOffice,
    DevelopmentOffice,
    DegreeOffice,
    StudentAffairsOffice,
    SuzhouOffice,
    DegreeCommittee,
    CompletionCertificates,
    StudentStatusChanges,
}

impl InfoPage {
    pub const ALL: &'static [InfoPage] = &[
        Self::GraduateSchoolIntroduction,
        Self::DepartmentLeaders,
        Self::GeneralOffice,
        Self::AdmissionsOffice,
        Self::TrainingOffice,
        Self::DevelopmentOffice,
        Self::DegreeOffice,
        Self::StudentAffairsOffice,
        Self::SuzhouOffice,
        Self::DegreeCommittee,
        Self::CompletionCertificates,
        Self::StudentStatusChanges,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::GraduateSchoolIntroduction => "机构简介：研究生院简介",
            Self::DepartmentLeaders => "机构简介：部门领导",
            Self::GeneralOffice => "机构简介：机构设置：综合办公室",
            Self::AdmissionsOffice => "机构简介：机构设置：招生办公室",
            Self::TrainingOffice => "机构简介：机构设置：培养办公室",
            Self::DevelopmentOffice => "机构简介：机构设置：发展办公室",
            Self::DegreeOffice => "机构简介：机构设置：学位办公室",
            Self::StudentAffairsOffice => "机构简介：机构设置：学籍与奖助管理办公室",
            Self::SuzhouOffice => "机构简介：机构设置：苏州校区办公室（苏州研究生院）",
            Self::DegreeCommittee => "学位与导师：学位评定委员会",
            Self::CompletionCertificates => "学籍与奖助管理：学籍学历管理：结业证书",
            Self::StudentStatusChanges => "学籍与奖助管理：学籍学历管理：学籍异动",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::GraduateSchoolIntroduction => "graduate-school-introduction",
            Self::DepartmentLeaders => "department-leaders",
            Self::GeneralOffice => "general-office",
            Self::AdmissionsOffice => "admissions-office",
            Self::TrainingOffice => "training-office",
            Self::DevelopmentOffice => "development-office",
            Self::DegreeOffice => "degree-office",
            Self::StudentAffairsOffice => "student-affairs-office",
            Self::SuzhouOffice => "suzhou-office",
            Self::DegreeCommittee => "degree-committee",
            Self::CompletionCertificates => "completion-certificates",
            Self::StudentStatusChanges => "student-status-changes",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            Self::GraduateSchoolIntroduction => "https://grawww.nju.edu.cn/55005/list.htm",
            Self::DepartmentLeaders => "https://grawww.nju.edu.cn/55006/list.htm",
            Self::GeneralOffice => "https://grawww.nju.edu.cn/zhbgs/list.htm",
            Self::AdmissionsOffice => "https://grawww.nju.edu.cn/30199/list.htm",
            Self::TrainingOffice => "https://grawww.nju.edu.cn/30200/list.htm",
            Self::DevelopmentOffice => "https://grawww.nju.edu.cn/61977/list.htm",
            Self::DegreeOffice => "https://grawww.nju.edu.cn/30202/list.htm",
            Self::StudentAffairsOffice => "https://grawww.nju.edu.cn/61978/list.htm",
            Self::SuzhouOffice => "https://grawww.nju.edu.cn/szxqbgs/list.htm",
            Self::DegreeCommittee => "https://grawww.nju.edu.cn/55021/list.htm",
            Self::CompletionCertificates => "https://grawww.nju.edu.cn/jyzs/list.htm",
            Self::StudentStatusChanges => "https://grawww.nju.edu.cn/xjyd/list.htm",
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
        .with_context(|| format!("failed to request graduate school {} list", section.title()))?
        .error_for_status()
        .with_context(|| {
            format!(
                "graduate school {} list returned an error status",
                section.title()
            )
        })?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate school {} list", section.title()))?;

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

pub async fn read_entry(client: &common::Client, title: &str, url: &str) -> Result<String> {
    let url = resolve_site_url(url)?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request graduate school entry: {url}"))?
        .error_for_status()
        .with_context(|| format!("graduate school entry returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();

    if !is_html_response(&content_type, &page_url) {
        return Ok(attachment_markdown(title, page_url.as_str()));
    }

    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate school entry: {url}"))?;
    article_html_to_markdown(&html, page_url.as_str())
}

pub async fn read_page(client: &common::Client, page: InfoPage) -> Result<String> {
    let url = page.url();
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to request graduate school page: {url}"))?
        .error_for_status()
        .with_context(|| format!("graduate school page returned an error status: {url}"))?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read graduate school page: {url}"))?;

    article_html_to_markdown(&html, page_url.as_str())
}

fn list_url(section: ArticleSection, page_index: u64) -> Result<Url> {
    if page_index == 0 {
        return Err(anyhow!("page index starts from 1"));
    }

    let path = if page_index == 1 {
        format!("{}/list.htm", section.list_path())
    } else {
        format!("{}/list{page_index}.htm", section.list_path())
    };

    Url::parse(SITE_BASE_URL)
        .context("invalid graduate school site base URL")?
        .join(&path)
        .with_context(|| format!("invalid graduate school list path: {path}"))
}

fn parse_article_list(
    section: ArticleSection,
    page_index: u64,
    page_url: &str,
    html: &str,
) -> Result<ArticlePage> {
    let document = Html::parse_document(html);
    let item_selector = selector(".col_news_list ul.news_list li.news")?;
    let title_selector = selector(".news_title a")?;
    let date_selector = selector(".news_meta")?;
    let month_day_selector = selector(".news_year")?;
    let year_selector = selector(".news_days")?;
    let page_size_selector = selector(".per_count")?;
    let total_selector = selector("em.all_count")?;
    let total_pages_selector = selector(".pages .all_pages")?;
    let base_url = Url::parse(page_url).with_context(|| format!("invalid page URL: {page_url}"))?;
    let mut articles = Vec::new();

    for item in document.select(&item_selector) {
        let Some(link) = item.select(&title_selector).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let mut url = base_url
            .join(href)
            .with_context(|| format!("invalid graduate school entry URL: {href}"))?;
        normalize_site_url(&mut url)?;
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
            .filter(|date| !date.is_empty())
            .or_else(|| {
                let month_day = item
                    .select(&month_day_selector)
                    .next()
                    .map(|date| text_content(date.text()))?;
                let year = item
                    .select(&year_selector)
                    .next()
                    .map(|date| text_content(date.text()))?;
                Some(format!("{year}-{month_day}"))
            })
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

fn resolve_site_url(url: &str) -> Result<Url> {
    let mut url = Url::parse(SITE_BASE_URL)
        .context("invalid graduate school site base URL")?
        .join(url)
        .with_context(|| format!("invalid graduate school URL: {url}"))?;
    normalize_site_url(&mut url)?;
    Ok(url)
}

fn normalize_site_url(url: &mut Url) -> Result<()> {
    if url.host_str() == Some("grawww.nju.edu.cn") && url.scheme() == "http" {
        url.set_scheme("https")
            .map_err(|_| anyhow!("failed to normalize graduate school URL: {url}"))?;
    }
    url.set_fragment(None);
    Ok(())
}

fn is_html_response(content_type: &str, url: &Url) -> bool {
    let content_type = content_type.to_ascii_lowercase();
    content_type.contains("text/html")
        || content_type.contains("application/xhtml+xml")
        || url.path().ends_with(".htm")
        || url.path().ends_with(".html")
}

fn attachment_markdown(title: &str, url: &str) -> String {
    format!("# {title}\n\n该条目是附件：[{title}]({url})")
}

fn article_html_to_markdown(html: &str, page_url: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let title = first_element(&document, &[".arti_title", ".col_title h2", "title"])?
        .map(|title| text_content(title.text()))
        .filter(|title| !title.is_empty());
    let content_html = first_element(
        &document,
        &[".wp_articlecontent", ".article", ".col_news_con"],
    )?
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

fn first_element<'a>(document: &'a Html, selectors: &[&str]) -> Result<Option<ElementRef<'a>>> {
    for css in selectors {
        let selector = selector(css)?;
        if let Some(element) = document.select(&selector).next() {
            return Ok(Some(element));
        }
    }
    Ok(None)
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
    let base_url =
        Url::parse(page_url).with_context(|| format!("invalid article page URL: {page_url}"))?;
    let pdf_player_selector = selector(".wp_pdf_player, [pdfsrc]")?;
    let mut urls = Vec::new();

    for element in document.select(&pdf_player_selector) {
        if let Some(pdf_src) = element.value().attr("pdfsrc") {
            push_unique_url(&mut urls, &base_url, pdf_src);
        }

        if let Some(src) = element.value().attr("src")
            && let Ok(viewer_url) = base_url.join(src)
            && let Some((_, file)) = viewer_url.query_pairs().find(|(name, _)| name == "file")
        {
            push_unique_url(&mut urls, &base_url, file.as_ref());
        }
    }

    Ok(urls)
}

fn push_unique_url(urls: &mut Vec<String>, base_url: &Url, url: &str) {
    if let Ok(mut url) = base_url.join(url) {
        let _ = normalize_site_url(&mut url);
        let url = url.to_string();
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn article_id(url: &Url) -> u64 {
    if let Some(id) = internal_article_id(url) {
        return id;
    }

    0x8000_0000_0000_0000 | stable_hash(url.as_str())
}

fn internal_article_id(url: &Url) -> Option<u64> {
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
    fn parses_only_main_article_list() {
        let html = r#"
            <div class="col_news_list">
                <ul class="news_list list2">
                    <li class="news">
                        <span class="news_year">08-31</span>
                        <span class="news_days">2026</span>
                        <span class="news_title"><a href="http://grawww.nju.edu.cn/d6/ef/c905a841455/page.htm" title="鼓楼校区综合服务大厅值班表">截断标题</a></span>
                    </li>
                </ul>
            </div>
            <div class="foot-center">
                <ul class="news_list"><li class="news"><span class="news_title"><a href="https://www.nju.edu.cn/">南京大学</a></span></li></ul>
            </div>
            <em class="per_count">14</em>
            <em class="all_count">771</em>
            <span class="pages"><em class="all_pages">56</em></span>
        "#;

        let page = parse_article_list(
            ArticleSection::Notifications,
            1,
            "https://grawww.nju.edu.cn/905/list.htm",
            html,
        )
        .unwrap();

        assert_eq!(page.page_size, Some(14));
        assert_eq!(page.total, Some(771));
        assert_eq!(page.total_pages, Some(56));
        assert_eq!(page.articles.len(), 1);
        assert_eq!(page.articles[0].id, 841455);
        assert_eq!(page.articles[0].title, "鼓楼校区综合服务大厅值班表");
        assert_eq!(page.articles[0].publish_time, "2026-08-31");
        assert_eq!(
            page.articles[0].url,
            "https://grawww.nju.edu.cn/d6/ef/c905a841455/page.htm"
        );
    }

    #[test]
    fn builds_later_page_url() {
        assert_eq!(
            list_url(ArticleSection::Notifications, 3).unwrap().as_str(),
            "https://grawww.nju.edu.cn/905/list3.htm"
        );
    }

    #[test]
    fn rejects_zero_page() {
        assert!(list_url(ArticleSection::Notifications, 0).is_err());
    }

    #[test]
    fn creates_attachment_markdown() {
        assert_eq!(
            attachment_markdown("申请表", "https://grawww.nju.edu.cn/form.doc"),
            "# 申请表\n\n该条目是附件：[申请表](https://grawww.nju.edu.cn/form.doc)"
        );
    }

    #[test]
    fn strips_duplicate_heading() {
        assert_eq!(strip_duplicate_heading("# 通知\n\n正文", "通知"), "正文");
    }
}
