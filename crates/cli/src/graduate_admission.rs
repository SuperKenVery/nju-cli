use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Subcommand, ValueEnum};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum GraduateAdmissionCommand {
    /// 列出支持的研究生招生网栏目。
    Columns,
    /// 列出栏目文章，并把文章 id 与 URL 缓存到本地。
    List {
        /// 栏目。
        #[arg(value_enum)]
        section: GraduateAdmissionSection,
        /// 页码，从 1 开始；传 --all 时忽略。
        #[arg(long, default_value_t = 1)]
        page: u64,
        /// 拉取栏目下所有文章；不传则只拉取指定页。
        #[arg(long)]
        all: bool,
    },
    /// 根据已缓存的文章 id 输出 Markdown 内容。
    View {
        /// 栏目。
        #[arg(value_enum)]
        section: GraduateAdmissionSection,
        /// 文章 id。需要先执行对应栏目的 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 输出单页栏目的 Markdown 内容，如联系招办、联系学院。
    Page {
        /// 栏目。
        #[arg(value_enum)]
        page: GraduateAdmissionPage,
    },
    /// 根据已缓存的文章 id 下载 Markdown 到目录。
    Download {
        /// 栏目。
        #[arg(value_enum)]
        section: GraduateAdmissionSection,
        /// 文章 id 列表；传 --all 时忽略该参数并下载栏目下所有文章。
        article_ids: Vec<u64>,
        /// 下载栏目下所有文章。
        #[arg(long)]
        all: bool,
        /// 输出目录；默认写到 nju-cli 缓存目录的 graduate-admission/<栏目>。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum GraduateAdmissionSection {
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

impl From<GraduateAdmissionSection> for graduate_admission::ArticleSection {
    fn from(section: GraduateAdmissionSection) -> Self {
        match section {
            GraduateAdmissionSection::MasterGuide => Self::MasterGuide,
            GraduateAdmissionSection::MasterNotifications => Self::MasterNotifications,
            GraduateAdmissionSection::DoctoralGuide => Self::DoctoralGuide,
            GraduateAdmissionSection::DoctoralNotifications => Self::DoctoralNotifications,
            GraduateAdmissionSection::SummerCampRecommendation => Self::SummerCampRecommendation,
            GraduateAdmissionSection::HongKongMacaoTaiwanGuide => Self::HongKongMacaoTaiwanGuide,
            GraduateAdmissionSection::HongKongMacaoTaiwanNotifications => {
                Self::HongKongMacaoTaiwanNotifications
            }
            GraduateAdmissionSection::PublicNotices => Self::PublicNotices,
            GraduateAdmissionSection::ScoreLines => Self::ScoreLines,
            GraduateAdmissionSection::AdmissionStatistics => Self::AdmissionStatistics,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum GraduateAdmissionPage {
    ContactOffice,
    ContactSchools,
}

impl GraduateAdmissionPage {
    fn title(self) -> &'static str {
        match self {
            Self::ContactOffice => "联系招办",
            Self::ContactSchools => "联系学院",
        }
    }

    fn url(self) -> &'static str {
        match self {
            Self::ContactOffice => "https://yzb.nju.edu.cn/lxzb/list.htm",
            Self::ContactSchools => "https://yzb.nju.edu.cn/lxyx/list.htm",
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

pub async fn handle(command: GraduateAdmissionCommand, client: &common::Client) -> Result<()> {
    match command {
        GraduateAdmissionCommand::Columns => print_columns(),
        GraduateAdmissionCommand::List { section, page, all } => {
            let section = graduate_admission::ArticleSection::from(section);
            let articles = list_articles(client, section, page, all)
                .await
                .with_context(|| format!("failed to list {}", section.title()))?;

            save_articles(section, &articles)?;
            print_articles(&articles);
        }
        GraduateAdmissionCommand::View {
            section,
            article_id,
        } => {
            let section = graduate_admission::ArticleSection::from(section);
            let article = find_cached_article(section, article_id)?;
            let markdown = graduate_admission::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read {} {article_id}", section.title()))?;

            println!("{markdown}");
        }
        GraduateAdmissionCommand::Page { page } => {
            let markdown = graduate_admission::read_page(client, page.url())
                .await
                .with_context(|| format!("failed to read {}", page.title()))?;

            println!("{markdown}");
        }
        GraduateAdmissionCommand::Download {
            section,
            article_ids,
            all,
            output_dir,
        } => {
            let section = graduate_admission::ArticleSection::from(section);
            let articles = articles_to_download(client, section, article_ids, all).await?;
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
    for section in graduate_admission::ArticleSection::ALL {
        println!(
            "{} {} {}",
            section.slug(),
            section.title(),
            section.list_url()
        );
    }
    println!("contact-office 联系招办 https://yzb.nju.edu.cn/lxzb/list.htm");
    println!("contact-schools 联系学院 https://yzb.nju.edu.cn/lxyx/list.htm");
}

async fn list_articles(
    client: &common::Client,
    section: graduate_admission::ArticleSection,
    page: u64,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    let articles = if all {
        graduate_admission::list_all_articles(client, section).await?
    } else {
        graduate_admission::get_articles(client, section, page.max(1))
            .await?
            .articles
    };

    Ok(cache_articles(articles))
}

async fn articles_to_download(
    client: &common::Client,
    section: graduate_admission::ArticleSection,
    article_ids: Vec<u64>,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    if all {
        let articles = list_articles(client, section, 1, true).await?;
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
                    "{} id {article_id} is not cached; run `nju-cli graduate-admission list {}` first",
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
    let markdown = graduate_admission::read_article(client, &article.url)
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

fn cache_articles(articles: Vec<graduate_admission::Article>) -> Vec<CachedArticle> {
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
    section: graduate_admission::ArticleSection,
    article_id: u64,
) -> Result<CachedArticle> {
    load_articles(section)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "{} id {article_id} is not cached; run `nju-cli graduate-admission list {}` first",
                section.title(),
                section.slug()
            )
        })
}

fn save_articles(
    section: graduate_admission::ArticleSection,
    articles: &[CachedArticle],
) -> Result<()> {
    let path = section_cache_file(section)?;
    let json = serde_json::to_string_pretty(articles)
        .context("failed to serialize cached graduate admission articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(section: graduate_admission::ArticleSection) -> Result<Vec<CachedArticle>> {
    let path = section_cache_file(section)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached {} articles from {}; run `nju-cli graduate-admission list {}` first",
            section.title(),
            path.display(),
            section.slug()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn section_cache_file(section: graduate_admission::ArticleSection) -> Result<PathBuf> {
    Ok(section_cache_dir(section)?.join("articles.json"))
}

fn section_cache_dir(section: graduate_admission::ArticleSection) -> Result<PathBuf> {
    let dir = graduate_admission_cache_dir()?.join(section.slug());
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn graduate_admission_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("graduate-admission");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
