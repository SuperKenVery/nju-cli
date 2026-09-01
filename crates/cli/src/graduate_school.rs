use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Subcommand, ValueEnum};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum GraduateSchoolCommand {
    /// 列出支持的研究生院文章栏目和静态页面。
    Columns,
    /// 列出栏目文章，并把文章 id 与 URL 缓存到本地。
    List {
        /// 文章栏目。
        #[arg(value_enum)]
        section: GraduateSchoolSection,
        /// 页码，从 1 开始；传 --all 时忽略。
        #[arg(long, default_value_t = 1)]
        page: u64,
        /// 拉取栏目下所有文章；不传则只拉取指定页。
        #[arg(long)]
        all: bool,
    },
    /// 根据已缓存的文章 id 输出 Markdown 内容或附件链接。
    View {
        /// 文章栏目。
        #[arg(value_enum)]
        section: GraduateSchoolSection,
        /// 文章 id。需要先执行对应栏目的 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 输出机构简介、部门办公室等静态页面的 Markdown 内容。
    Page {
        /// 静态页面。
        #[arg(value_enum)]
        page: GraduateSchoolPage,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum GraduateSchoolSection {
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

impl From<GraduateSchoolSection> for graduate_school::ArticleSection {
    fn from(section: GraduateSchoolSection) -> Self {
        match section {
            GraduateSchoolSection::Notifications => Self::Notifications,
            GraduateSchoolSection::ServiceGuides => Self::ServiceGuides,
            GraduateSchoolSection::TrainingRegulations => Self::TrainingRegulations,
            GraduateSchoolSection::NationalSponsoredPrograms => Self::NationalSponsoredPrograms,
            GraduateSchoolSection::UniversityExchangePrograms => Self::UniversityExchangePrograms,
            GraduateSchoolSection::TrainingInnovationProjects => Self::TrainingInnovationProjects,
            GraduateSchoolSection::CombinedMasterDoctor => Self::CombinedMasterDoctor,
            GraduateSchoolSection::CourseDevelopment => Self::CourseDevelopment,
            GraduateSchoolSection::TrainingStudentStatus => Self::TrainingStudentStatus,
            GraduateSchoolSection::TrainingDownloads => Self::TrainingDownloads,
            GraduateSchoolSection::DegreeWork => Self::DegreeWork,
            GraduateSchoolSection::SupervisorWork => Self::SupervisorWork,
            GraduateSchoolSection::ExcellentTheses => Self::ExcellentTheses,
            GraduateSchoolSection::DegreeImprovementProgram => Self::DegreeImprovementProgram,
            GraduateSchoolSection::DegreeDownloads => Self::DegreeDownloads,
            GraduateSchoolSection::HonoraryDoctors => Self::HonoraryDoctors,
            GraduateSchoolSection::StudentPolicies => Self::StudentPolicies,
            GraduateSchoolSection::GraduationCertificates => Self::GraduationCertificates,
            GraduateSchoolSection::StudentDownloads => Self::StudentDownloads,
            GraduateSchoolSection::StudentRegistration => Self::StudentRegistration,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum GraduateSchoolPage {
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

impl From<GraduateSchoolPage> for graduate_school::InfoPage {
    fn from(page: GraduateSchoolPage) -> Self {
        match page {
            GraduateSchoolPage::GraduateSchoolIntroduction => Self::GraduateSchoolIntroduction,
            GraduateSchoolPage::DepartmentLeaders => Self::DepartmentLeaders,
            GraduateSchoolPage::GeneralOffice => Self::GeneralOffice,
            GraduateSchoolPage::AdmissionsOffice => Self::AdmissionsOffice,
            GraduateSchoolPage::TrainingOffice => Self::TrainingOffice,
            GraduateSchoolPage::DevelopmentOffice => Self::DevelopmentOffice,
            GraduateSchoolPage::DegreeOffice => Self::DegreeOffice,
            GraduateSchoolPage::StudentAffairsOffice => Self::StudentAffairsOffice,
            GraduateSchoolPage::SuzhouOffice => Self::SuzhouOffice,
            GraduateSchoolPage::DegreeCommittee => Self::DegreeCommittee,
            GraduateSchoolPage::CompletionCertificates => Self::CompletionCertificates,
            GraduateSchoolPage::StudentStatusChanges => Self::StudentStatusChanges,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedArticle {
    id: u64,
    title: String,
    url: String,
    #[serde(default)]
    publish_time: String,
}

pub async fn handle(command: GraduateSchoolCommand, client: &common::Client) -> Result<()> {
    match command {
        GraduateSchoolCommand::Columns => print_columns(),
        GraduateSchoolCommand::List { section, page, all } => {
            let section = graduate_school::ArticleSection::from(section);
            let articles = list_articles(client, section, page, all)
                .await
                .with_context(|| format!("failed to list {}", section.title()))?;

            save_articles(section, &articles)?;
            print_articles(&articles);
        }
        GraduateSchoolCommand::View {
            section,
            article_id,
        } => {
            let section = graduate_school::ArticleSection::from(section);
            let article = find_cached_article(section, article_id)?;
            let markdown = graduate_school::read_entry(client, &article.title, &article.url)
                .await
                .with_context(|| format!("failed to read {} {article_id}", section.title()))?;

            println!("{markdown}");
        }
        GraduateSchoolCommand::Page { page } => {
            let page = graduate_school::InfoPage::from(page);
            let markdown = graduate_school::read_page(client, page)
                .await
                .with_context(|| format!("failed to read {}", page.title()))?;

            println!("{markdown}");
        }
    }

    Ok(())
}

fn print_columns() {
    for section in graduate_school::ArticleSection::ALL {
        println!(
            "list {} {} {}",
            section.slug(),
            section.title(),
            section.list_url()
        );
    }
    for page in graduate_school::InfoPage::ALL {
        println!("page {} {} {}", page.slug(), page.title(), page.url());
    }
}

async fn list_articles(
    client: &common::Client,
    section: graduate_school::ArticleSection,
    page: u64,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    let articles = if all {
        graduate_school::list_all_articles(client, section).await?
    } else {
        graduate_school::get_articles(client, section, page)
            .await?
            .articles
    };

    Ok(cache_articles(articles))
}

fn print_articles(articles: &[CachedArticle]) {
    for article in articles {
        if article.publish_time.is_empty() {
            println!("{} {}", article.id, article.title);
        } else {
            println!("{} {} {}", article.id, article.publish_time, article.title);
        }
    }
}

fn cache_articles(articles: Vec<graduate_school::Article>) -> Vec<CachedArticle> {
    articles
        .into_iter()
        .map(|article| CachedArticle {
            id: article.id,
            title: article.title,
            url: article.url,
            publish_time: article.publish_time,
        })
        .collect()
}

fn find_cached_article(
    section: graduate_school::ArticleSection,
    article_id: u64,
) -> Result<CachedArticle> {
    load_articles(section)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "{} id {article_id} is not cached; run `nju-cli graduate-school list {}` first",
                section.title(),
                section.slug()
            )
        })
}

fn save_articles(
    section: graduate_school::ArticleSection,
    articles: &[CachedArticle],
) -> Result<()> {
    let path = section_cache_file(section)?;
    let json = serde_json::to_string_pretty(articles)
        .context("failed to serialize cached graduate school articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(section: graduate_school::ArticleSection) -> Result<Vec<CachedArticle>> {
    let path = section_cache_file(section)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached {} articles from {}; run `nju-cli graduate-school list {}` first",
            section.title(),
            path.display(),
            section.slug()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn section_cache_file(section: graduate_school::ArticleSection) -> Result<PathBuf> {
    let dir = graduate_school_cache_dir()?.join(section.slug());
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir.join("articles.json"))
}

fn graduate_school_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("graduate-school");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
