use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum ExchangeSystemCommand {
    /// 新闻通知。
    Notice {
        #[command(subcommand)]
        command: NoticeCommand,
    },
    /// 交换项目。
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
}

#[derive(Debug, Args)]
pub struct ListOptions {
    /// 每页数量。
    #[arg(long, default_value_t = 20)]
    page_size: u64,
}

#[derive(Debug, Subcommand)]
pub enum NoticeCommand {
    /// 列出新闻通知，并把通知 id 与正文缓存到本地。
    List(ListOptions),
    /// 根据已缓存的通知 id 输出 Markdown 内容。
    View { notice_id: u64 },
    /// 根据已缓存的通知 id 下载 Markdown 到缓存目录。
    Download { notice_ids: Vec<u64> },
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommand {
    /// 列出交换项目，并把项目 id 与描述缓存到本地。
    List(ListOptions),
    /// 根据已缓存的项目 id 输出 Markdown 描述。
    View { project_id: u64 },
    /// 根据已缓存的项目 id 下载 Markdown 描述到缓存目录。
    Download { project_ids: Vec<u64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedNotice {
    id: u64,
    title: String,
    html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedProject {
    id: u64,
    title: String,
    description_html: Option<String>,
    requirement_url: Option<String>,
    remark: Option<String>,
}

pub async fn handle(command: ExchangeSystemCommand, client: &common::Client) -> Result<()> {
    match command {
        ExchangeSystemCommand::Notice { command } => handle_notice(command, client).await,
        ExchangeSystemCommand::Project { command } => handle_project(command, client).await,
    }
}

async fn handle_notice(command: NoticeCommand, client: &common::Client) -> Result<()> {
    match command {
        NoticeCommand::List(options) => {
            let page = exchange_system::get_notices(client, 1, options.page_size)
                .await
                .context("failed to list exchange system notices")?;
            let notices = page
                .data
                .into_iter()
                .map(|notice| CachedNotice {
                    id: notice.pid,
                    title: notice.bt,
                    html: notice.nr,
                })
                .collect::<Vec<_>>();

            save_notices(&notices)?;

            for notice in notices {
                println!("{} {}", notice.id, notice.title);
            }
            Ok(())
        }
        NoticeCommand::View { notice_id } => {
            let notice = find_cached_notice(notice_id)?;
            let markdown =
                common::html_to_markdown_with_base_url(&notice.html, exchange_base_url())
                    .with_context(|| format!("failed to convert notice {notice_id} to markdown"))?;

            println!("{markdown}");
            Ok(())
        }
        NoticeCommand::Download { notice_ids } => {
            if notice_ids.is_empty() {
                return Err(anyhow!("please provide at least one notice id"));
            }

            let notices = load_notices()?
                .into_iter()
                .map(|notice| (notice.id, notice))
                .collect::<HashMap<_, _>>();
            let dir = exchange_system_cache_dir()?.join("notices");
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for notice_id in notice_ids {
                let notice = notices.get(&notice_id).ok_or_else(|| {
                    anyhow!(
                        "notice id {notice_id} is not cached; run `nju-cli exchange-system notice list --page-size 100` first"
                    )
                })?;
                let markdown =
                    common::html_to_markdown_with_base_url(&notice.html, exchange_base_url())
                        .with_context(|| {
                            format!("failed to convert notice {notice_id} to markdown")
                        })?;
                let path = dir.join(format!("{}.md", sanitize_filename::sanitize(&notice.title)));

                std::fs::write(&path, markdown)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                println!("{} {}", notice_id, path.display());
            }
            Ok(())
        }
    }
}

async fn handle_project(command: ProjectCommand, client: &common::Client) -> Result<()> {
    match command {
        ProjectCommand::List(options) => {
            let page = exchange_system::get_projects(client, 1, options.page_size)
                .await
                .context("failed to list exchange system projects")?;
            let projects = page
                .data
                .into_iter()
                .map(|project| CachedProject {
                    id: project.pid,
                    title: project.mc,
                    description_html: project.xmms,
                    requirement_url: project.sqyq,
                    remark: project.bz,
                })
                .collect::<Vec<_>>();

            save_projects(&projects)?;

            for project in projects {
                println!("{} {}", project.id, project.title);
            }
            Ok(())
        }
        ProjectCommand::View { project_id } => {
            let project = find_cached_project(project_id)?;
            let markdown = cached_project_to_markdown(&project)
                .with_context(|| format!("failed to convert project {project_id} to markdown"))?;

            println!("{markdown}");
            Ok(())
        }
        ProjectCommand::Download { project_ids } => {
            if project_ids.is_empty() {
                return Err(anyhow!("please provide at least one project id"));
            }

            let projects = load_projects()?
                .into_iter()
                .map(|project| (project.id, project))
                .collect::<HashMap<_, _>>();
            let dir = exchange_system_cache_dir()?.join("projects");
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for project_id in project_ids {
                let project = projects.get(&project_id).ok_or_else(|| {
                    anyhow!(
                        "project id {project_id} is not cached; run `nju-cli exchange-system project list --page-size 100` first"
                    )
                })?;
                let markdown = cached_project_to_markdown(project).with_context(|| {
                    format!("failed to convert project {project_id} to markdown")
                })?;
                let path = dir.join(format!(
                    "{}.md",
                    sanitize_filename::sanitize(&project.title)
                ));

                std::fs::write(&path, markdown)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                println!("{} {}", project_id, path.display());
            }
            Ok(())
        }
    }
}

fn cached_project_to_markdown(project: &CachedProject) -> Result<String> {
    let mut html = String::new();

    if let Some(description) = &project.description_html {
        html.push_str(description);
    }
    if let Some(requirement_url) = &project.requirement_url {
        html.push_str("<p>申请要求：<a href=\"");
        html.push_str(requirement_url);
        html.push_str("\">查看申请要求</a></p>");
    }
    if let Some(remark) = &project.remark {
        html.push_str("<h2>备注</h2>");
        html.push_str(remark);
    }

    common::html_to_markdown_with_base_url(&html, exchange_base_url())
}

fn find_cached_notice(notice_id: u64) -> Result<CachedNotice> {
    load_notices()?.into_iter().find(|notice| notice.id == notice_id).ok_or_else(|| {
        anyhow!(
            "notice id {notice_id} is not cached; run `nju-cli exchange-system notice list --page-size 100` first"
        )
    })
}

fn find_cached_project(project_id: u64) -> Result<CachedProject> {
    load_projects()?.into_iter().find(|project| project.id == project_id).ok_or_else(|| {
        anyhow!(
            "project id {project_id} is not cached; run `nju-cli exchange-system project list --page-size 100` first"
        )
    })
}

fn save_notices(notices: &[CachedNotice]) -> Result<()> {
    let path = notices_cache_file()?;
    let json =
        serde_json::to_string_pretty(notices).context("failed to serialize cached notices")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn save_projects(projects: &[CachedProject]) -> Result<()> {
    let path = projects_cache_file()?;
    let json =
        serde_json::to_string_pretty(projects).context("failed to serialize cached projects")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_notices() -> Result<Vec<CachedNotice>> {
    let path = notices_cache_file()?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached notices from {}; run `nju-cli exchange-system notice list --page-size 100` first",
            path.display()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn load_projects() -> Result<Vec<CachedProject>> {
    let path = projects_cache_file()?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached projects from {}; run `nju-cli exchange-system project list --page-size 100` first",
            path.display()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn notices_cache_file() -> Result<PathBuf> {
    Ok(exchange_system_cache_dir()?.join("notices.json"))
}

fn projects_cache_file() -> Result<PathBuf> {
    Ok(exchange_system_cache_dir()?.join("projects.json"))
}

fn exchange_system_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("exchange-system");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn exchange_base_url() -> &'static str {
    "http://elite.nju.edu.cn/exchangesystem/index/"
}
