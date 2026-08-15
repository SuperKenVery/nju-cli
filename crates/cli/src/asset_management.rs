use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::Subcommand;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum AssetManagementCommand {
    /// 列出支持的资产管理处栏目。
    Columns,
    /// 列出栏目文章，并把文章 id 与 URL 缓存到本地。
    List {
        /// 栏目。
        #[arg(value_parser = parse_section)]
        section: asset_management::ArticleSection,
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
        #[arg(value_parser = parse_section)]
        section: asset_management::ArticleSection,
        /// 文章 id。需要先执行对应栏目的 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 根据已缓存的文章 id 下载 Markdown 到目录。
    Download {
        /// 栏目。
        #[arg(value_parser = parse_section)]
        section: asset_management::ArticleSection,
        /// 文章 id 列表；传 --all 时忽略该参数并下载栏目下所有文章。
        article_ids: Vec<u64>,
        /// 下载栏目下所有文章。
        #[arg(long)]
        all: bool,
        /// 输出目录；默认写到 nju-cli 缓存目录的 asset-management/<栏目>。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedArticle {
    id: u64,
    title: String,
    url: String,
    #[serde(default)]
    publish_date: String,
}

pub async fn handle(command: AssetManagementCommand, client: &common::Client) -> Result<()> {
    match command {
        AssetManagementCommand::Columns => print_columns(),
        AssetManagementCommand::List { section, page, all } => {
            let articles = list_articles(client, section, page, all)
                .await
                .with_context(|| format!("failed to list {}", section.title()))?;

            save_articles(section, &articles)?;
            print_articles(&articles);
        }
        AssetManagementCommand::View {
            section,
            article_id,
        } => {
            let article = find_cached_article(section, article_id)?;
            let markdown = asset_management::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read {} {article_id}", section.title()))?;

            println!("{markdown}");
        }
        AssetManagementCommand::Download {
            section,
            article_ids,
            all,
            output_dir,
        } => {
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

fn parse_section(value: &str) -> Result<asset_management::ArticleSection, String> {
    asset_management::ArticleSection::from_slug(value).ok_or_else(|| {
        let supported = asset_management::ArticleSection::ALL
            .iter()
            .map(|section| section.slug())
            .collect::<Vec<_>>()
            .join(", ");
        format!("unsupported asset management section `{value}`; supported: {supported}")
    })
}

fn print_columns() {
    for section in asset_management::ArticleSection::ALL {
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
    section: asset_management::ArticleSection,
    page: u64,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    let articles = if all {
        asset_management::list_all_articles(client, section).await?
    } else {
        asset_management::get_articles(client, section, page.max(1))
            .await?
            .articles
    };

    Ok(cache_articles(articles))
}

async fn articles_to_download(
    client: &common::Client,
    section: asset_management::ArticleSection,
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
                    "{} id {article_id} is not cached; run `nju-cli asset-management list {}` first",
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
    let markdown = asset_management::read_article(client, &article.url)
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
        if article.publish_date.is_empty() {
            println!("{} {}", article.id, article.title);
        } else {
            println!("{} {} {}", article.id, article.publish_date, article.title);
        }
    }
}

fn cache_articles(articles: Vec<asset_management::Article>) -> Vec<CachedArticle> {
    articles
        .into_iter()
        .map(|article| CachedArticle {
            id: article.id,
            title: article.title,
            url: article.url,
            publish_date: article.publish_date,
        })
        .collect()
}

fn find_cached_article(
    section: asset_management::ArticleSection,
    article_id: u64,
) -> Result<CachedArticle> {
    load_articles(section)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "{} id {article_id} is not cached; run `nju-cli asset-management list {}` first",
                section.title(),
                section.slug()
            )
        })
}

fn save_articles(
    section: asset_management::ArticleSection,
    articles: &[CachedArticle],
) -> Result<()> {
    let path = section_cache_file(section)?;
    let json = serde_json::to_string_pretty(articles)
        .context("failed to serialize cached asset management articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(section: asset_management::ArticleSection) -> Result<Vec<CachedArticle>> {
    let path = section_cache_file(section)?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached {} articles from {}; run `nju-cli asset-management list {}` first",
            section.title(),
            path.display(),
            section.slug()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn section_cache_file(section: asset_management::ArticleSection) -> Result<PathBuf> {
    Ok(section_cache_dir(section)?.join("articles.json"))
}

fn section_cache_dir(section: asset_management::ArticleSection) -> Result<PathBuf> {
    let dir = asset_management_cache_dir()?.join(section.slug());
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn asset_management_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("asset-management");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
