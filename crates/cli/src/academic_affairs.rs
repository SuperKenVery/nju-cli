use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use academic_affairs::{ArticleColumn, DownloadFile, article::Article};
use anyhow::{Context, Result, anyhow};
use clap::Subcommand;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum AcademicAffairsCommand {
    /// 输出教务网当前全学年校历的 PDF 和图片链接。
    Calendar,
    /// 下载专区：校历目录、各类表格和各类模板。
    Downloads {
        #[command(subcommand)]
        command: DownloadZoneCommand,
    },
    /// 教务网公告通知。
    Notifications {
        #[command(subcommand)]
        command: NotificationCommand,
    },
    /// 部门领导和机构设置。
    Institutions {
        #[command(subcommand)]
        command: InstitutionCommand,
    },
    /// 教育部文件。
    #[command(name = "ministry-of-edu-doc")]
    MinistryOfEduDoc {
        #[command(subcommand)]
        command: ArticleCollectionCommand,
    },
    /// 学生手册。
    #[command(name = "students-manual")]
    StudentsManual {
        #[command(subcommand)]
        command: ArticleCollectionCommand,
    },
    /// 教师手册。
    #[command(name = "teachers-manual")]
    TeachersManual {
        #[command(subcommand)]
        command: ArticleCollectionCommand,
    },
    /// 学校文件。
    #[command(name = "school-regulations")]
    SchoolRegulations {
        #[command(subcommand)]
        command: ArticleCollectionCommand,
    },
    /// 办事流程。
    #[command(name = "admin-procedures")]
    AdminProcedures {
        #[command(subcommand)]
        command: ArticleCollectionCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum NotificationCommand {
    /// 列出教务网公告通知，并把公告 id 与 URL 缓存到本地。
    List {
        /// 页码，从 1 开始。
        #[arg(long, default_value_t = 1)]
        page: u64,
        /// 每页公告数量。
        #[arg(long, default_value_t = 20)]
        page_size: u64,
    },
    /// 根据已缓存的公告 id 输出公告 Markdown 内容。
    View {
        /// 公告 id。需要先执行 notifications list 以缓存 id 与 URL。
        announcement_id: u64,
    },
    /// 根据已缓存的公告 id 下载公告 Markdown 到缓存目录。
    Download {
        /// 公告 id 列表。需要先执行 notifications list 以缓存 id 与 URL。
        announcement_ids: Vec<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ArticleCollectionCommand {
    /// 列出文章，并把 id 与 URL 缓存到本地。
    List {
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
        /// 文章 id。需要先执行 list 以缓存 id 与 URL。
        article_id: u64,
    },
    /// 下载文章 Markdown 到目录。
    Download {
        /// 文章 id 列表；传 --all 时忽略该参数并下载栏目下所有文章。
        article_ids: Vec<u64>,
        /// 下载栏目下所有文章。
        #[arg(long)]
        all: bool,
        /// 拉取所有文章时每页文章数量。
        #[arg(long, default_value_t = 100)]
        page_size: u64,
        /// 输出目录；默认写到 nju-cli 缓存目录的 academic-affairs/<栏目>。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum DownloadZoneCommand {
    /// 校历目录。
    #[command(name = "calendar-catalog")]
    CalendarCatalog {
        #[command(subcommand)]
        command: DownloadResourceCommand,
    },
    /// 各类表格。
    Forms {
        #[command(subcommand)]
        command: DownloadResourceCommand,
    },
    /// 各类模板。
    Templates {
        #[command(subcommand)]
        command: DownloadResourceCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum DownloadResourceCommand {
    /// 列出下载专区条目，并把 id 与 URL 缓存到本地。
    List {
        /// 页码，从 1 开始；传 --all 时忽略。
        #[arg(long, default_value_t = 1)]
        page: u64,
        /// 每页条目数量。
        #[arg(long, default_value_t = 100)]
        page_size: u64,
        /// 拉取栏目下所有条目；不传则只拉取指定页。
        #[arg(long)]
        all: bool,
    },
    /// 下载条目对应的实际附件文件。
    Download {
        /// 条目 id 列表；传 --all 时忽略该参数并下载栏目下所有条目。
        resource_ids: Vec<u64>,
        /// 下载栏目下所有条目。
        #[arg(long)]
        all: bool,
        /// 拉取所有条目时每页数量。
        #[arg(long, default_value_t = 100)]
        page_size: u64,
        /// 输出目录；默认写到 nju-cli 缓存目录的 academic-affairs/<栏目>。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum InstitutionCommand {
    /// 显示部门领导，并列出所有机构。
    List,
    /// 查看机构详情 Markdown 内容。
    View {
        /// 机构栏目 id 或文章 id。需要先执行 institutions list 以缓存 id 与 URL。
        institution_id: u64,
    },
    /// 下载机构详情 Markdown 到目录。
    Download {
        /// 机构栏目 id 或文章 id 列表。需要先执行 institutions list 以缓存 id 与 URL。
        institution_ids: Vec<u64>,
        /// 输出目录；默认写到 nju-cli 缓存目录的 academic-affairs/institutions。
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedArticle {
    id: u64,
    #[serde(default)]
    column_id: Option<u64>,
    title: String,
    url: String,
    #[serde(default)]
    publish_time: String,
}

pub async fn handle(command: AcademicAffairsCommand, client: &common::Client) -> Result<()> {
    match command {
        AcademicAffairsCommand::Calendar => handle_calendar(client).await?,
        AcademicAffairsCommand::Downloads { command } => {
            handle_download_zone(client, command).await?
        }
        AcademicAffairsCommand::Notifications { command } => {
            handle_notifications(client, command).await?
        }
        AcademicAffairsCommand::Institutions { command } => {
            handle_institutions(client, command).await?
        }
        AcademicAffairsCommand::MinistryOfEduDoc { command } => {
            handle_article_collection(client, ArticleColumn::MinistryDocuments, command).await?
        }
        AcademicAffairsCommand::StudentsManual { command } => {
            handle_article_collection(client, ArticleColumn::StudentHandbook, command).await?
        }
        AcademicAffairsCommand::TeachersManual { command } => {
            handle_article_collection(client, ArticleColumn::TeacherHandbook, command).await?
        }
        AcademicAffairsCommand::SchoolRegulations { command } => {
            handle_article_collection(client, ArticleColumn::SchoolDocuments, command).await?
        }
        AcademicAffairsCommand::AdminProcedures { command } => {
            handle_article_collection(client, ArticleColumn::Procedures, command).await?
        }
    }

    Ok(())
}

async fn handle_calendar(client: &common::Client) -> Result<()> {
    let calendar = academic_affairs::get_calendar(client)
        .await
        .context("failed to get academic calendar")?;

    println!("{}", calendar.title);
    println!("页面：{}", calendar.page_url);
    println!("PDF：");
    for url in calendar.pdf_urls {
        println!("{url}");
    }
    println!("图片：");
    for url in calendar.image_urls {
        println!("{url}");
    }

    Ok(())
}

async fn handle_notifications(client: &common::Client, command: NotificationCommand) -> Result<()> {
    match command {
        NotificationCommand::List { page, page_size } => {
            let page = academic_affairs::get_announcements(client, page, page_size)
                .await
                .context("failed to list academic affairs notifications")?;
            let announcements = cache_articles(page.data);

            save_articles(&announcements, announcements_cache_file()?)?;
            print_articles(&announcements);
        }
        NotificationCommand::View { announcement_id } => {
            let announcement =
                find_cached_article(announcement_id, announcements_cache_file()?, "notification")?;
            let markdown = academic_affairs::read_announcement(client, &announcement.url)
                .await
                .with_context(|| format!("failed to read notification {announcement_id}"))?;

            println!("{markdown}");
        }
        NotificationCommand::Download { announcement_ids } => {
            if announcement_ids.is_empty() {
                return Err(anyhow!("please provide at least one notification id"));
            }

            let announcements = load_articles(announcements_cache_file()?, "notifications")?
                .into_iter()
                .map(|announcement| (announcement.id, announcement))
                .collect::<HashMap<_, _>>();
            let dir = academic_affairs_cache_dir()?;

            for announcement_id in announcement_ids {
                let announcement = announcements.get(&announcement_id).ok_or_else(|| {
                    anyhow!(
                        "notification id {announcement_id} is not cached; run `nju-cli academic-affairs notifications list --page-size 100` first"
                    )
                })?;
                download_article(client, announcement, &dir)
                    .await
                    .with_context(|| {
                        format!("failed to download notification {announcement_id}")
                    })?;
            }
        }
    }

    Ok(())
}

async fn handle_article_collection(
    client: &common::Client,
    column: ArticleColumn,
    command: ArticleCollectionCommand,
) -> Result<()> {
    match command {
        ArticleCollectionCommand::List {
            page,
            page_size,
            all,
        } => {
            let articles = list_articles(client, column, page, page_size, all)
                .await
                .with_context(|| format!("failed to list {}", column.title()))?;
            save_articles(&articles, column_cache_file(column)?)?;
            print_articles(&articles);
        }
        ArticleCollectionCommand::View { article_id } => {
            let article =
                find_cached_article(article_id, column_cache_file(column)?, column.title())?;
            let markdown = academic_affairs::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read {} {article_id}", column.title()))?;

            println!("{markdown}");
        }
        ArticleCollectionCommand::Download {
            article_ids,
            all,
            page_size,
            output_dir,
        } => {
            let articles =
                articles_to_download(client, column, article_ids, all, page_size).await?;
            let dir = output_dir.unwrap_or(column_cache_dir(column)?);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for article in &articles {
                download_article(client, article, &dir)
                    .await
                    .with_context(|| {
                        format!("failed to download {} {}", column.title(), article.id)
                    })?;
            }
        }
    }

    Ok(())
}

async fn handle_download_zone(client: &common::Client, command: DownloadZoneCommand) -> Result<()> {
    match command {
        DownloadZoneCommand::CalendarCatalog { command } => {
            handle_download_resources(client, ArticleColumn::CalendarCatalog, command).await?
        }
        DownloadZoneCommand::Forms { command } => {
            handle_download_resources(client, ArticleColumn::Forms, command).await?
        }
        DownloadZoneCommand::Templates { command } => {
            handle_download_resources(client, ArticleColumn::Templates, command).await?
        }
    }

    Ok(())
}

async fn handle_download_resources(
    client: &common::Client,
    column: ArticleColumn,
    command: DownloadResourceCommand,
) -> Result<()> {
    match command {
        DownloadResourceCommand::List {
            page,
            page_size,
            all,
        } => {
            let articles = list_articles(client, column, page, page_size, all)
                .await
                .with_context(|| format!("failed to list {}", column.title()))?;
            save_articles(&articles, column_cache_file(column)?)?;
            print_articles(&articles);
        }
        DownloadResourceCommand::Download {
            resource_ids,
            all,
            page_size,
            output_dir,
        } => {
            let resources =
                articles_to_download(client, column, resource_ids, all, page_size).await?;
            let dir = output_dir.unwrap_or(column_cache_dir(column)?);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for resource in &resources {
                download_resource_files(client, resource, &dir)
                    .await
                    .with_context(|| {
                        format!("failed to download {} {}", column.title(), resource.id)
                    })?;
            }
        }
    }

    Ok(())
}

async fn handle_institutions(client: &common::Client, command: InstitutionCommand) -> Result<()> {
    let column = ArticleColumn::Institutions;

    match command {
        InstitutionCommand::List => {
            print_department_leaders(client).await?;
            println!("\n## 机构设置\n");

            let institutions = academic_affairs::get_institutions(client)
                .await
                .context("failed to list institutions")?;
            let articles = cache_institutions(institutions);
            save_articles(&articles, column_cache_file(column)?)?;
            print_institutions(&articles);
        }
        InstitutionCommand::View { institution_id } => {
            let article = find_cached_institution(institution_id)?;
            let markdown = academic_affairs::read_article(client, &article.url)
                .await
                .with_context(|| format!("failed to read institution {institution_id}"))?;

            println!("{markdown}");
        }
        InstitutionCommand::Download {
            institution_ids,
            output_dir,
        } => {
            if institution_ids.is_empty() {
                return Err(anyhow!("please provide at least one institution id"));
            }

            let articles = load_articles(column_cache_file(column)?, column.title())?;
            let dir = output_dir.unwrap_or(column_cache_dir(column)?);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;

            for institution_id in institution_ids {
                let article = articles
                    .iter()
                    .find(|article| matches_cached_institution(article, institution_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "institution id {institution_id} is not cached; run `nju-cli academic-affairs institutions list` first"
                        )
                    })?;
                download_article(client, article, &dir)
                    .await
                    .with_context(|| format!("failed to download institution {institution_id}"))?;
            }
        }
    }

    Ok(())
}

async fn print_department_leaders(client: &common::Client) -> Result<()> {
    let article = first_column_article(client, ArticleColumn::DepartmentLeaders)
        .await
        .context("failed to get department leaders page")?;
    let markdown = academic_affairs::read_article(client, &article.url)
        .await
        .context("failed to read department leaders")?;

    println!("{markdown}");

    Ok(())
}

async fn list_articles(
    client: &common::Client,
    column: ArticleColumn,
    page: u64,
    page_size: u64,
    all: bool,
) -> Result<Vec<CachedArticle>> {
    let articles = if all {
        academic_affairs::list_all_column_articles(client, column, page_size).await?
    } else {
        academic_affairs::get_column_articles(client, column, page.max(1), page_size)
            .await?
            .data
    };

    Ok(cache_articles(articles))
}

async fn articles_to_download(
    client: &common::Client,
    column: ArticleColumn,
    article_ids: Vec<u64>,
    all: bool,
    page_size: u64,
) -> Result<Vec<CachedArticle>> {
    if all {
        let articles = list_articles(client, column, 1, page_size, true).await?;
        save_articles(&articles, column_cache_file(column)?)?;
        return Ok(articles);
    }

    if article_ids.is_empty() {
        return Err(anyhow!(
            "please provide at least one article id or pass --all to download all {}",
            column.title()
        ));
    }

    let articles = load_articles(column_cache_file(column)?, column.title())?
        .into_iter()
        .map(|article| (article.id, article))
        .collect::<HashMap<_, _>>();

    article_ids
        .into_iter()
        .map(|article_id| {
            articles.get(&article_id).cloned().ok_or_else(|| {
                anyhow!(
                    "{} id {article_id} is not cached; run `nju-cli academic-affairs {} list` first",
                    column.title(),
                    command_hint(column)
                )
            })
        })
        .collect()
}

async fn first_column_article(client: &common::Client, column: ArticleColumn) -> Result<Article> {
    academic_affairs::get_column_articles(client, column, 1, 1)
        .await
        .with_context(|| format!("failed to list {}", column.title()))?
        .data
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no article found in {}", column.title()))
}

async fn download_article(
    client: &common::Client,
    article: &CachedArticle,
    dir: &std::path::Path,
) -> Result<()> {
    let markdown = academic_affairs::read_article(client, &article.url)
        .await
        .with_context(|| format!("failed to read article {}", article.id))?;
    let file_name = format!("{}.md", sanitize_filename::sanitize(&article.title));
    let path = dir.join(file_name);

    std::fs::write(&path, markdown)
        .with_context(|| format!("failed to write {}", path.display()))?;
    println!("{} {}", article.id, path.display());

    Ok(())
}

async fn download_resource_files(
    client: &common::Client,
    article: &CachedArticle,
    dir: &std::path::Path,
) -> Result<()> {
    let files = academic_affairs::list_article_download_files(client, &article.url)
        .await
        .with_context(|| format!("failed to list downloadable files for {}", article.id))?;
    if files.is_empty() {
        return Err(anyhow!("no downloadable file found for {}", article.id));
    }

    let target_dir = if files.len() > 1 {
        dir.join(sanitize_filename::sanitize(&article.title))
    } else {
        dir.to_path_buf()
    };
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    let mut used_file_names = HashSet::new();
    for (index, file) in files.iter().enumerate() {
        let file_name = resource_file_name(article, file, index, files.len());
        let file_name = unique_file_name(file_name, &mut used_file_names);
        let path = target_dir.join(file_name);
        let bytes = client
            .get(&file.url)
            .send()
            .await
            .with_context(|| format!("failed to request {}", file.url))?
            .error_for_status()
            .with_context(|| format!("download returned an error status: {}", file.url))?
            .bytes()
            .await
            .with_context(|| format!("failed to read {}", file.url))?;

        std::fs::write(&path, bytes)
            .with_context(|| format!("failed to write {}", path.display()))?;
        println!("{} {}", article.id, path.display());
    }

    Ok(())
}

fn resource_file_name(
    article: &CachedArticle,
    file: &DownloadFile,
    index: usize,
    total: usize,
) -> String {
    let title = if total == 1 {
        article.title.clone()
    } else if file.title.is_empty() {
        format!("{}-{}", article.title, index + 1)
    } else {
        file.title.clone()
    };
    let mut file_name = sanitize_filename::sanitize(title);
    if file_name.trim().is_empty() {
        file_name = format!("download-{}", index + 1);
    }

    if std::path::Path::new(&file_name).extension().is_none() {
        if let Some(extension) = file_extension(&file.title).or_else(|| url_extension(&file.url)) {
            file_name.push('.');
            file_name.push_str(&extension);
        }
    }

    file_name
}

fn unique_file_name(file_name: String, used_file_names: &mut HashSet<String>) -> String {
    if used_file_names.insert(file_name.clone()) {
        return file_name;
    }

    let path = std::path::Path::new(&file_name);
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("download");
    let extension = path.extension().and_then(|extension| extension.to_str());

    for index in 2.. {
        let candidate = match extension {
            Some(extension) => format!("{stem}-{index}.{extension}"),
            None => format!("{stem}-{index}"),
        };
        if used_file_names.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("unique file name loop should always return")
}

fn file_extension(file_name: &str) -> Option<String> {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .map(str::to_string)
}

fn url_extension(url: &str) -> Option<String> {
    let url = reqwest::Url::parse(url).ok()?;
    url.path_segments()?
        .next_back()?
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .map(str::to_string)
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

fn print_institutions(articles: &[CachedArticle]) {
    for article in articles {
        let column_id = article.column_id.unwrap_or(article.id);
        println!("{} {} {}", column_id, article.id, article.title);
    }
}

fn cache_articles(articles: Vec<Article>) -> Vec<CachedArticle> {
    articles.into_iter().map(cache_article).collect()
}

fn cache_institutions(institutions: Vec<academic_affairs::Institution>) -> Vec<CachedArticle> {
    institutions
        .into_iter()
        .map(|institution| CachedArticle {
            id: institution.article.id,
            column_id: Some(institution.column_id),
            title: institution.title,
            url: institution.article.url,
            publish_time: institution.article.publish_time,
        })
        .collect()
}

fn cache_article(article: Article) -> CachedArticle {
    CachedArticle {
        id: article.id,
        column_id: None,
        title: article.title,
        url: article.url,
        publish_time: article.publish_time,
    }
}

fn find_cached_article(article_id: u64, path: PathBuf, label: &str) -> Result<CachedArticle> {
    load_articles(path, label)?
        .into_iter()
        .find(|article| article.id == article_id)
        .ok_or_else(|| {
            anyhow!(
                "{label} id {article_id} is not cached; run the corresponding list command first"
            )
        })
}

fn find_cached_institution(institution_id: u64) -> Result<CachedArticle> {
    load_articles(column_cache_file(ArticleColumn::Institutions)?, "institutions")?
        .into_iter()
        .find(|article| matches_cached_institution(article, institution_id))
        .ok_or_else(|| {
            anyhow!(
                "institution id {institution_id} is not cached; run `nju-cli academic-affairs institutions list` first"
            )
        })
}

fn matches_cached_institution(article: &CachedArticle, institution_id: u64) -> bool {
    article.id == institution_id || article.column_id == Some(institution_id)
}

fn save_articles(articles: &[CachedArticle], path: PathBuf) -> Result<()> {
    let json =
        serde_json::to_string_pretty(articles).context("failed to serialize cached articles")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_articles(path: PathBuf, label: &str) -> Result<Vec<CachedArticle>> {
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached {label} from {}; run the corresponding list command first",
            path.display()
        )
    })?;

    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn announcements_cache_file() -> Result<PathBuf> {
    Ok(academic_affairs_cache_dir()?.join("announcements.json"))
}

fn column_cache_file(column: ArticleColumn) -> Result<PathBuf> {
    Ok(column_cache_dir(column)?.join("articles.json"))
}

fn column_cache_dir(column: ArticleColumn) -> Result<PathBuf> {
    let dir = academic_affairs_cache_dir()?.join(column_cache_name(column));
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn academic_affairs_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("academic-affairs");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}

fn column_cache_name(column: ArticleColumn) -> &'static str {
    match column {
        ArticleColumn::MinistryDocuments => "ministry-documents",
        ArticleColumn::SchoolDocuments => "school-documents",
        ArticleColumn::StudentHandbook => "student-handbook",
        ArticleColumn::TeacherHandbook => "teacher-handbook",
        ArticleColumn::Procedures => "procedures",
        ArticleColumn::DepartmentLeaders => "department-leaders",
        ArticleColumn::Institutions => "institutions",
        ArticleColumn::CalendarCatalog => "calendar-catalog",
        ArticleColumn::Forms => "forms",
        ArticleColumn::Templates => "templates",
    }
}

fn command_hint(column: ArticleColumn) -> &'static str {
    match column {
        ArticleColumn::MinistryDocuments => "ministry-of-edu-doc",
        ArticleColumn::SchoolDocuments => "school-regulations",
        ArticleColumn::StudentHandbook => "students-manual",
        ArticleColumn::TeacherHandbook => "teachers-manual",
        ArticleColumn::Procedures => "admin-procedures",
        ArticleColumn::Institutions => "institutions",
        ArticleColumn::DepartmentLeaders => "institutions",
        ArticleColumn::CalendarCatalog => "downloads calendar-catalog",
        ArticleColumn::Forms => "downloads forms",
        ArticleColumn::Templates => "downloads templates",
    }
}
