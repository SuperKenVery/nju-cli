use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Subcommand, ValueEnum};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum ItscCommand {
    /// 列出 ITSC 服务说明或正版软件教程，并把页面 id 与 URL 缓存到本地。
    List {
        /// 栏目：services 为各类服务说明，licensed-software 为正版软件安装教程。
        #[arg(value_enum)]
        section: ItscSection,
        /// 递归抓取正文中的同站子页面。正版软件教程建议开启。
        #[arg(long)]
        recursive: bool,
        /// 递归抓取时最多请求的页面数。
        #[arg(long, default_value_t = 200)]
        max_pages: usize,
    },
    /// 根据已缓存的页面 id 输出 Markdown 内容。
    View {
        /// 页面 id。需要先执行 list 以缓存 id 与 URL。
        page_id: u64,
    },
    /// 根据已缓存的页面 id 下载 Markdown 到缓存目录。
    Download {
        /// 页面 id 列表；传 --all 时忽略该参数。
        page_ids: Vec<u64>,
        /// 下载当前缓存中的所有页面。
        #[arg(long)]
        all: bool,
        /// 输出目录；默认写到 nju-cli 缓存目录的 itsc/pages。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ItscSection {
    Services,
    LicensedSoftware,
}

impl From<ItscSection> for itsc::Section {
    fn from(section: ItscSection) -> Self {
        match section {
            ItscSection::Services => itsc::Section::Services,
            ItscSection::LicensedSoftware => itsc::Section::LicensedSoftware,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPage {
    id: u64,
    title: String,
    url: String,
    kind: itsc::PageKind,
}

pub async fn handle(command: ItscCommand, client: &common::Client) -> Result<()> {
    match command {
        ItscCommand::List {
            section,
            recursive,
            max_pages,
        } => {
            let section = itsc::Section::from(section);
            let pages = itsc::list_pages(client, section, recursive, max_pages)
                .await
                .with_context(|| format!("failed to list {}", section.title()))?
                .into_iter()
                .map(cache_page)
                .collect::<Vec<_>>();

            save_pages(&pages)?;
            print_pages(&pages);
        }
        ItscCommand::View { page_id } => {
            let page = find_cached_page(page_id)?;
            let markdown = itsc::read_page(client, &page.url)
                .await
                .with_context(|| format!("failed to read ITSC page {page_id}"))?;

            println!("{markdown}");
        }
        ItscCommand::Download {
            page_ids,
            all,
            output_dir,
        } => {
            let pages = pages_to_download(page_ids, all)?;
            let dir = output_dir.unwrap_or(itsc_pages_cache_dir()?);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for page in pages {
                let markdown = itsc::read_page(client, &page.url)
                    .await
                    .with_context(|| format!("failed to read ITSC page {}", page.id))?;
                let file_name = format!("{}.md", sanitize_filename::sanitize(&page.title));
                let path = dir.join(file_name);

                std::fs::write(&path, markdown)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                println!("{} {}", page.id, path.display());
            }
        }
    }

    Ok(())
}

fn cache_page(page: itsc::Page) -> CachedPage {
    CachedPage {
        id: page.id,
        title: page.title,
        url: page.url,
        kind: page.kind,
    }
}

fn print_pages(pages: &[CachedPage]) {
    for page in pages {
        println!("{} {:?} {}", page.id, page.kind, page.title);
    }
}

fn pages_to_download(page_ids: Vec<u64>, all: bool) -> Result<Vec<CachedPage>> {
    let pages = load_pages()?;
    if all {
        return Ok(pages);
    }
    if page_ids.is_empty() {
        return Err(anyhow!(
            "please provide at least one ITSC page id or pass --all"
        ));
    }

    let pages_by_id = pages
        .into_iter()
        .map(|page| (page.id, page))
        .collect::<HashMap<_, _>>();

    page_ids
        .into_iter()
        .map(|page_id| {
            pages_by_id.get(&page_id).cloned().ok_or_else(|| {
                anyhow!("ITSC page id {page_id} is not cached; run `nju-cli itsc list services --recursive` or `nju-cli itsc list licensed-software --recursive` first")
            })
        })
        .collect()
}

fn find_cached_page(page_id: u64) -> Result<CachedPage> {
    load_pages()?
        .into_iter()
        .find(|page| page.id == page_id)
        .ok_or_else(|| {
            anyhow!(
                "ITSC page id {page_id} is not cached; run `nju-cli itsc list services --recursive` or `nju-cli itsc list licensed-software --recursive` first"
            )
        })
}

fn save_pages(pages: &[CachedPage]) -> Result<()> {
    let path = pages_cache_file()?;
    let json =
        serde_json::to_string_pretty(pages).context("failed to serialize cached ITSC pages")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_pages() -> Result<Vec<CachedPage>> {
    let path = pages_cache_file()?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached ITSC pages from {}; run an `nju-cli itsc list ...` command first",
            path.display()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn pages_cache_file() -> Result<PathBuf> {
    Ok(itsc_cache_dir()?.join("pages.json"))
}

fn itsc_pages_cache_dir() -> Result<PathBuf> {
    let dir = itsc_cache_dir()?.join("pages");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn itsc_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("itsc");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
