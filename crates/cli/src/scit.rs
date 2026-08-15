use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Subcommand, ValueEnum};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum ScitCommand {
    /// 列出支持的科学技术研究院栏目。
    Columns,
    /// 列出栏目文章，并把文章 id 与 URL 缓存到本地。
    List {
        /// 栏目。
        #[arg(value_enum)]
        section: ScitSection,
        /// 页码，从 1 开始；传 --all 时忽略。
        #[arg(long, default_value_t = 1)]
        page: u64,
        /// 每页文章数量。
        #[arg(long, default_value_t = 100)]
        page_size: u64,
        /// 拉取栏目下所有文章；不传则只拉取指定页。
        #[arg(long)]
        all: bool,
    },
    /// 根据已缓存的文章 id 输出 Markdown 内容。
    View {
        /// 栏目。
        #[arg(value_enum)]
        section: ScitSection,
        /// 文章 id。需要先执行对应栏目的 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 根据已缓存的文章 id 下载 Markdown 到目录。
    Download {
        /// 栏目。
        #[arg(value_enum)]
        section: ScitSection,
        /// 文章 id 列表；传 --all 时忽略该参数并下载栏目下所有文章。
        article_ids: Vec<u64>,
        /// 下载栏目下所有文章。
        #[arg(long)]
        all: bool,
        /// 拉取所有文章时每页数量。
        #[arg(long, default_value_t = 100)]
        page_size: u64,
        /// 输出目录；默认写到 nju-cli 缓存目录的 scit/<栏目>。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum ScitSection {
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

impl From<ScitSection> for scit::ArticleSection {
    fn from(section: ScitSection) -> Self {
        match section {
            ScitSection::Notifications => Self::Notifications,
            ScitSection::ResearchNews => Self::ResearchNews,
            ScitSection::PublicInfo => Self::PublicInfo,
            ScitSection::Ai => Self::Ai,
            ScitSection::ResearchProjects => Self::ResearchProjects,
            ScitSection::ResearchProjectsNsfc => Self::ResearchProjectsNsfc,
            ScitSection::ResearchProjectsMost => Self::ResearchProjectsMost,
            ScitSection::ResearchProjectsMoe => Self::ResearchProjectsMoe,
            ScitSection::ResearchProjectsJiangsu => Self::ResearchProjectsJiangsu,
            ScitSection::ResearchProjectsFundamental => Self::ResearchProjectsFundamental,
            ScitSection::ResearchProjectsIndustry => Self::ResearchProjectsIndustry,
            ScitSection::ResearchProjectsOther => Self::ResearchProjectsOther,
            ScitSection::Workflow => Self::Workflow,
            ScitSection::WorkflowProjects => Self::WorkflowProjects,
            ScitSection::WorkflowPlatforms => Self::WorkflowPlatforms,
            ScitSection::WorkflowAchievements => Self::WorkflowAchievements,
            ScitSection::WorkflowContracts => Self::WorkflowContracts,
            ScitSection::WorkflowOther => Self::WorkflowOther,
            ScitSection::Downloads => Self::Downloads,
            ScitSection::DownloadsProjects => Self::DownloadsProjects,
            ScitSection::DownloadsPlatforms => Self::DownloadsPlatforms,
            ScitSection::DownloadsAchievements => Self::DownloadsAchievements,
            ScitSection::DownloadsContracts => Self::DownloadsContracts,
            ScitSection::DownloadsOther => Self::DownloadsOther,
            ScitSection::Institutions => Self::Institutions,
            ScitSection::InstitutionsLeaders => Self::InstitutionsLeaders,
            ScitSection::InstitutionsGeneralOffice => Self::InstitutionsGeneralOffice,
            ScitSection::InstitutionsPlatformOffice => Self::InstitutionsPlatformOffice,
            ScitSection::InstitutionsVerticalProjectsOffice => {
                Self::InstitutionsVerticalProjectsOffice
            }
            ScitSection::InstitutionsIndustryOffice => Self::InstitutionsIndustryOffice,
            ScitSection::InstitutionsAchievementsOffice => Self::InstitutionsAchievementsOffice,
            ScitSection::InstitutionsSpecialOffice => Self::InstitutionsSpecialOffice,
            ScitSection::InstitutionsSuzhouOffice => Self::InstitutionsSuzhouOffice,
            ScitSection::Policies => Self::Policies,
            ScitSection::PoliciesProjects => Self::PoliciesProjects,
            ScitSection::PoliciesPlatforms => Self::PoliciesPlatforms,
            ScitSection::PoliciesAchievements => Self::PoliciesAchievements,
            ScitSection::PoliciesContracts => Self::PoliciesContracts,
            ScitSection::PoliciesOther => Self::PoliciesOther,
            ScitSection::Platforms => Self::Platforms,
            ScitSection::PlatformsNational => Self::PlatformsNational,
            ScitSection::PlatformsMoe => Self::PlatformsMoe,
            ScitSection::PlatformsJiangsu => Self::PlatformsJiangsu,
            ScitSection::PlatformsOtherMinistry => Self::PlatformsOtherMinistry,
            ScitSection::PlatformsUniversity => Self::PlatformsUniversity,
            ScitSection::PlatformsIndustry => Self::PlatformsIndustry,
            ScitSection::PlatformsNotices => Self::PlatformsNotices,
            ScitSection::Achievements => Self::Achievements,
            ScitSection::AchievementsAwards => Self::AchievementsAwards,
            ScitSection::AchievementsPapers => Self::AchievementsPapers,
            ScitSection::AchievementsIp => Self::AchievementsIp,
            ScitSection::AcademicIntegrity => Self::AcademicIntegrity,
            ScitSection::AcademicIntegrityPolicies => Self::AcademicIntegrityPolicies,
            ScitSection::AcademicIntegrityReports => Self::AcademicIntegrityReports,
            ScitSection::AcademicIntegrityStories => Self::AcademicIntegrityStories,
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

pub async fn handle(command: ScitCommand, client: &common::Client) -> Result<()> {
    match command {
        ScitCommand::Columns => print_columns(),
        ScitCommand::List {
            section,
            page,
            page_size,
            all,
        } => {
            let section = scit::ArticleSection::from(section);
            let articles = list_articles(client, section, page, page_size, all)
                .await
                .with_context(|| format!("failed to list {}", section.title()))?;

            save_articles(section, &articles)?;
            print_articles(&articles);
        }
        ScitCommand::View {
            section,
            article_id,
        } => {
            let section = scit::ArticleSection::from(section);
            let article = find_cached_article(section, article_id)?;
            let markdown = scit::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read {} {article_id}", section.title()))?;

            println!("{markdown}");
        }
        ScitCommand::Download {
            section,
            article_ids,
            all,
            page_size,
            output_dir,
        } => {
            let section = scit::ArticleSection::from(section);
            let articles =
                articles_to_download(client, section, article_ids, all, page_size).await?;
            let dir = output_dir.unwrap_or(section_cache_dir(section)?);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for article in &articles {
                download_article(client, article, &dir)
                    .await
                    .with_context(|| {
                        format!("failed to download {} {}", section.title(), article.id)
                    })?;
            }
        }
    }

    Ok(())
}

fn print_columns() {
    for section in scit::ArticleSection::ALL {
        println!(
            "{} {} {}",
            section.slug(),
            section.title(),
            section.list_url()
        );
    }
}

async fn list_articles(
    client: &common::Client,
    section: scit::ArticleSection,
    page: u64,
    page_size: u64,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    let articles = if all {
        scit::list_all_articles(client, section, page_size).await?
    } else {
        scit::get_articles(client, section, page.max(1), page_size)
            .await?
            .data
    };

    Ok(cache_articles(articles))
}

async fn articles_to_download(
    client: &common::Client,
    section: scit::ArticleSection,
    article_ids: Vec<u64>,
    all: bool,
    page_size: u64,
) -> Result<Vec<CachedArticle>> {
    if all {
        let articles = list_articles(client, section, 1, page_size, true).await?;
        save_articles(section, &articles)?;
        return Ok(articles);
    }

    if article_ids.is_empty() {
        return Err(anyhow!(
            "please provide at least one article id or pass --all to download all {}",
            section.title()
        ));
    }

    let articles = load_articles(section)?
        .into_iter()
        .map(|article| (article.id, article))
        .collect::<HashMap<_, _>>();

    article_ids
        .into_iter()
        .map(|article_id| {
            articles.get(&article_id).cloned().ok_or_else(|| {
                anyhow!(
                    "{} id {article_id} is not cached; run `nju-cli scit list {}` first",
                    section.title(),
                    section.slug()
                )
            })
        })
        .collect()
}

async fn download_article(
    client: &common::Client,
    article: &CachedArticle,
    dir: &std::path::Path,
) -> Result<()> {
    let markdown = scit::read_article(client, &article.url)
        .await
        .with_context(|| format!("failed to read article {}", article.id))?;
    let file_name = format!("{}.md", sanitize_filename::sanitize(&article.title));
    let path = dir.join(file_name);

    std::fs::write(&path, markdown)
        .with_context(|| format!("failed to write {}", path.display()))?;
    println!("{} {}", article.id, path.display());

    Ok(())
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

fn cache_articles(articles: Vec<scit::Article>) -> Vec<CachedArticle> {
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

fn find_cached_article(section: scit::ArticleSection, article_id: u64) -> Result<CachedArticle> {
    load_articles(section)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "{} id {article_id} is not cached; run `nju-cli scit list {}` first",
                section.title(),
                section.slug()
            )
        })
}

fn save_articles(section: scit::ArticleSection, articles: &[CachedArticle]) -> Result<()> {
    let path = section_cache_file(section)?;
    let json = serde_json::to_string_pretty(articles)
        .context("failed to serialize cached SCIT articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(section: scit::ArticleSection) -> Result<Vec<CachedArticle>> {
    let path = section_cache_file(section)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached {} articles from {}; run `nju-cli scit list {}` first",
            section.title(),
            path.display(),
            section.slug()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn section_cache_file(section: scit::ArticleSection) -> Result<PathBuf> {
    Ok(section_cache_dir(section)?.join("articles.json"))
}

fn section_cache_dir(section: scit::ArticleSection) -> Result<PathBuf> {
    let dir = scit_cache_dir()?.join(section.slug());
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn scit_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("scit");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
