use anyhow::Result;
use clap::{Parser, Subcommand};

mod academic_affairs;
mod asset_management;
mod auth;
mod download;
mod ehall;
mod exchange_system;
mod graduate_admission;
mod itsc;
mod scit;
mod venue;
mod youth_league;

#[derive(Debug, Parser)]
#[command(name = "nju-cli")]
#[command(about = "南京大学站点命令行工具")]
struct Cli {
    /// 通过南京大学 Web VPN 发送所有网页请求。
    #[arg(long, global = true)]
    vpn: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// 登录统一认证并缓存 CASTGC cookie。
    Login(auth::LoginCommand),
    /// 登录南京大学 Web VPN；首次调用发送短信，第二次通过 --sms-code 提交验证码。
    #[command(name = "login-vpn")]
    LoginVpn(auth::VpnLoginCommand),
    /// 读取 HTML 页面并转换为 Markdown。
    #[command(name = "view-html")]
    ViewHtml {
        /// 要读取的 HTML 页面 URL。
        url: String,
    },
    /// 下载 URL 指向的文件；配合 --vpn 可携带 Web VPN 会话下载附件。
    Download(download::DownloadCommand),
    /// 需要 ehall 登录态的服务。
    Ehall {
        #[command(subcommand)]
        command: ehall::EhallCommand,
    },
    /// 教务网公告通知。
    #[command(name = "academic-affairs")]
    AcademicAffairs {
        #[command(subcommand)]
        command: academic_affairs::AcademicAffairsCommand,
    },
    /// 资产管理处：新闻、通知、规章、下载、处罚、指南和公开招租。
    #[command(name = "asset-management")]
    AssetManagement {
        #[command(subcommand)]
        command: asset_management::AssetManagementCommand,
    },
    /// 交换生系统新闻通知和项目。
    #[command(name = "exchange-system")]
    ExchangeSystem {
        #[command(subcommand)]
        command: exchange_system::ExchangeSystemCommand,
    },
    /// 研究生招生网：硕士、博士、推免、港澳台招生和信息公开。
    #[command(name = "graduate-admission")]
    GraduateAdmission {
        #[command(subcommand)]
        command: graduate_admission::GraduateAdmissionCommand,
    },
    /// 南大团委最新动态和公告通知。
    #[command(name = "youth-league")]
    YouthLeague {
        #[command(subcommand)]
        command: youth_league::YouthLeagueCommand,
    },
    /// 信息化中心服务说明和正版软件安装教程。
    Itsc {
        #[command(subcommand)]
        command: itsc::ItscCommand,
    },
    /// 科学技术研究院文章。
    Scit {
        #[command(subcommand)]
        command: scit::ScitCommand,
    },
    /// 体育场馆预约、查询和订单管理。
    Venue {
        #[command(subcommand)]
        command: venue::VenueCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let direct_mode = if cli.vpn {
        auth::ClientMode::WebVpn
    } else {
        auth::ClientMode::Direct
    };
    let authenticated_mode = if cli.vpn {
        auth::ClientMode::WebVpn
    } else {
        auth::ClientMode::Authenticated
    };

    match cli.command {
        Command::Login(command) => auth::login(command).await?,
        Command::LoginVpn(command) => auth::login_vpn(command).await?,
        Command::ViewHtml { url } => {
            let client = auth::get_client(direct_mode).await?;
            let markdown = common::read_html_page(&client, &url).await?;
            println!("{markdown}");
        }
        Command::Download(command) => {
            let client = auth::get_client(direct_mode).await?;
            download::handle(command, &client).await?;
        }
        Command::Ehall { command } => {
            let client = auth::get_client(authenticated_mode).await?;
            ehall::handle(command, &client).await?
        }
        Command::AcademicAffairs { command } => {
            let client = auth::get_client(direct_mode).await?;
            academic_affairs::handle(command, &client).await?
        }
        Command::AssetManagement { command } => {
            let client = auth::get_client(direct_mode).await?;
            asset_management::handle(command, &client).await?
        }
        Command::ExchangeSystem { command } => {
            let client = auth::get_client(authenticated_mode).await?;
            exchange_system::handle(command, &client).await?
        }
        Command::GraduateAdmission { command } => {
            let client = auth::get_client(direct_mode).await?;
            graduate_admission::handle(command, &client).await?
        }
        Command::YouthLeague { command } => {
            let client = auth::get_client(direct_mode).await?;
            youth_league::handle(command, &client).await?
        }
        Command::Itsc { command } => {
            let client = auth::get_client(direct_mode).await?;
            itsc::handle(command, &client).await?
        }
        Command::Scit { command } => {
            let client = auth::get_client(direct_mode).await?;
            scit::handle(command, &client).await?
        }
        Command::Venue { command } => {
            let client = auth::get_client(authenticated_mode).await?;
            venue::handle(command, &client).await?
        }
    }

    Ok(())
}
