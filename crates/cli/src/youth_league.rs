use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Subcommand, ValueEnum};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum YouthLeagueCommand {
    /// 列出南大团委文章，并把文章 id 与 URL 缓存到本地。
    List {
        /// 栏目：latest-dynamics 为最新动态，announcements 为公告通知。
        #[arg(value_enum)]
        section: YouthLeagueSection,
        /// 页码，从 1 开始。团委站点固定每页 14 条。
        #[arg(long, default_value_t = 1)]
        page: u64,
    },
    /// 根据已缓存的文章 id 输出 Markdown 内容。
    View {
        /// 栏目：latest-dynamics 为最新动态，announcements 为公告通知。
        #[arg(value_enum)]
        section: YouthLeagueSection,
        /// 文章 id。需要先执行 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 根据已缓存的文章 id 下载 Markdown 到缓存目录。
    Download {
        /// 栏目：latest-dynamics 为最新动态，announcements 为公告通知。
        #[arg(value_enum)]
        section: YouthLeagueSection,
        /// 文章 id 列表。需要先执行 list 以缓存 id 与 URL。
        article_ids: Vec<u64>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum YouthLeagueSection {
    LatestDynamics,
    Announcements,
}

impl From<YouthLeagueSection> for youth_league::ArticleSection {
    fn from(section: YouthLeagueSection) -> Self {
        match section {
            YouthLeagueSection::LatestDynamics => youth_league::ArticleSection::LatestDynamics,
            YouthLeagueSection::Announcements => youth_league::ArticleSection::Announcements,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedArticle {
    id: u64,
    title: String,
    publish_date: String,
    url: String,
}

pub async fn handle(command: YouthLeagueCommand, client: &common::Client) -> Result<()> {
    match command {
        YouthLeagueCommand::List { section, page } => {
            let section = youth_league::ArticleSection::from(section);
            let page = youth_league::get_articles(client, section, page)
                .await
                .with_context(|| format!("failed to list youth league {}", section.title()))?;
            let articles = page
                .articles
                .into_iter()
                .map(|article| CachedArticle {
                    id: article.id,
                    title: article.title,
                    publish_date: article.publish_date,
                    url: article.url,
                })
                .collect::<Vec<_>>();

            save_articles(section, &articles)?;

            for article in articles {
                println!("{} {} {}", article.id, article.publish_date, article.title);
            }
        }
        YouthLeagueCommand::View {
            section,
            article_id,
        } => {
            let section = youth_league::ArticleSection::from(section);
            let article = find_cached_article(section, article_id)?;
            let markdown = youth_league::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read youth league article {article_id}"))?;

            println!("{markdown}");
        }
        YouthLeagueCommand::Download {
            section,
            article_ids,
        } => {
            if article_ids.is_empty() {
                return Err(anyhow!("please provide at least one article id"));
            }

            let section = youth_league::ArticleSection::from(section);
            let articles = load_articles(section)?
                .into_iter()
                .map(|article| (article.id, article))
                .collect::<HashMap<_, _>>();
            let dir = youth_league_section_cache_dir(section)?;

            for article_id in article_ids {
                let article = articles.get(&article_id).ok_or_else(|| {
                    anyhow!(
                        "article id {article_id} is not cached; run `nju-cli youth-league list {} --page 1` first",
                        section.slug()
                    )
                })?;
                let markdown = youth_league::read_article(client, &article.url)
                    .await
                    .with_context(|| format!("failed to read youth league article {article_id}"))?;
                let file_name = format!("{}.md", sanitize_filename::sanitize(&article.title));
                let path = dir.join(file_name);

                std::fs::write(&path, markdown)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                println!("{} {}", article_id, path.display());
            }
        }
    }

    Ok(())
}

fn find_cached_article(
    section: youth_league::ArticleSection,
    article_id: u64,
) -> Result<CachedArticle> {
    load_articles(section)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "article id {article_id} is not cached; run `nju-cli youth-league list {} --page 1` first",
                section.slug()
            )
        })
}

fn save_articles(section: youth_league::ArticleSection, articles: &[CachedArticle]) -> Result<()> {
    let path = articles_cache_file(section)?;
    let json = serde_json::to_string_pretty(articles)
        .context("failed to serialize cached youth league articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(section: youth_league::ArticleSection) -> Result<Vec<CachedArticle>> {
    let path = articles_cache_file(section)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached youth league articles from {}; run `nju-cli youth-league list {} --page 1` first",
            path.display(),
            section.slug()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn articles_cache_file(section: youth_league::ArticleSection) -> Result<PathBuf> {
    Ok(youth_league_cache_dir()?.join(format!("{}.json", section.slug())))
}

fn youth_league_section_cache_dir(section: youth_league::ArticleSection) -> Result<PathBuf> {
    let dir = youth_league_cache_dir()?.join(section.slug());

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn youth_league_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("youth-league");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
