use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use anyhow::{Context, Result, anyhow, ensure};
use clap::Args;
use nju_web_vpn::{LoginOperation, WebVpnMiddleware};
use platform_dirs::AppDirs;
use reqwest::{Url, cookie::Jar};
use serde::{Deserialize, Serialize};

const AUTH_LOGIN_URL: &str = "https://authserver.nju.edu.cn/authserver/login";
const VPN_ENTRY_URL: &str = "https://ehall.nju.edu.cn/";

#[derive(Debug, Args)]
pub struct LoginCommand {
    /// 统一认证用户名。也可使用 NJU_USERNAME 环境变量。
    #[arg(long)]
    username: Option<String>,
    /// 统一认证密码。也可使用 NJU_PASSWORD 环境变量。
    #[arg(long)]
    password: Option<String>,
    /// 直接保存已有 CASTGC cookie。
    #[arg(long)]
    castgc: Option<String>,
}

#[derive(Debug, Args)]
pub struct VpnLoginCommand {
    /// 手机收到的短信验证码。首次调用不传该参数以发送验证码。
    #[arg(long)]
    sms_code: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ClientMode {
    Direct,
    Authenticated,
    WebVpn,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedAuth {
    castgc: String,
    updated_at_unix: u64,
}

pub async fn login(command: LoginCommand) -> Result<()> {
    let castgc = match command.castgc {
        Some(castgc) => castgc,
        None => {
            let username = command
                .username
                .or_else(|| std::env::var("NJU_USERNAME").ok())
                .ok_or_else(|| anyhow!("please provide --username or NJU_USERNAME"))?;
            let password = command
                .password
                .or_else(|| std::env::var("NJU_PASSWORD").ok())
                .ok_or_else(|| anyhow!("please provide --password or NJU_PASSWORD"))?;
            nju_unified_auth::login(username, password)
                .await
                .context("failed to login NJU unified auth")?
        }
    };

    save_castgc(castgc)?;
    println!("login saved");

    Ok(())
}

/// 开始或继续 Web VPN 短信登录。
pub async fn login_vpn(command: VpnLoginCommand) -> Result<()> {
    match command.sms_code {
        None => {
            let castgc = load_castgc().context(
                "no unified auth login found; run `nju-cli login` before `nju-cli login-vpn`",
            )?;
            let operation = LoginOperation::start(
                Url::parse(VPN_ENTRY_URL).context("invalid Web VPN entry URL")?,
                castgc,
            )
            .await
            .context("failed to start Web VPN login")?;
            save_vpn_operation(&vpn_pending_file()?, &operation)?;
            println!("SMS verification code sent; rerun `nju-cli login-vpn --sms-code CODE`");
        }
        Some(code) => {
            let pending_path = vpn_pending_file()?;
            let mut operation = load_vpn_operation(&pending_path).with_context(|| {
                format!(
                    "no pending Web VPN login found; run `nju-cli login-vpn` first ({})",
                    pending_path.display()
                )
            })?;
            operation
                .submit_sms(code)
                .await
                .context("failed to complete Web VPN login")?;
            save_vpn_operation(&vpn_session_file()?, &operation)?;
            std::fs::remove_file(&pending_path).with_context(|| {
                format!(
                    "Web VPN login succeeded, but failed to remove {}",
                    pending_path.display()
                )
            })?;
            println!("Web VPN login saved");
        }
    }

    Ok(())
}

/// 按访问方式构建业务请求使用的 client。
pub async fn get_client(mode: ClientMode) -> Result<common::Client> {
    let client = match mode {
        ClientMode::Direct => reqwest::Client::builder()
            .user_agent("nju-cli")
            .build()
            .context("failed to build reqwest client")?,
        ClientMode::Authenticated => authenticated_client().await?,
        ClientMode::WebVpn => {
            let path = vpn_session_file()?;
            let operation = load_vpn_operation(&path).with_context(|| {
                format!(
                    "not logged in to Web VPN; run `nju-cli login-vpn` first ({})",
                    path.display()
                )
            })?;
            ensure!(
                operation.sid.is_some(),
                "Web VPN login is incomplete; run `nju-cli login-vpn` again"
            );

            return Ok(common::ClientBuilder::new(operation.client)
                .with(WebVpnMiddleware)
                .build());
        }
    };

    Ok(common::ClientBuilder::new(client).build())
}

/// 构建带缓存统一认证登陆态的 client，并检查登陆态仍然有效。
async fn authenticated_client() -> Result<reqwest::Client> {
    let castgc = load_castgc()?;
    let jar = Arc::new(Jar::default());
    jar.add_cookie_str(
        &format!("CASTGC={castgc}"),
        &Url::parse("https://authserver.nju.edu.cn").context("invalid NJU authserver URL")?,
    );
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .user_agent("nju-cli")
        .build()
        .context("failed to build authenticated reqwest client")?;

    ensure_logged_in(&client).await?;

    Ok(client)
}

/// 检查 client 是否持有有效的统一认证登录态。
async fn ensure_logged_in(client: &reqwest::Client) -> Result<()> {
    let login_page = client
        .get(AUTH_LOGIN_URL)
        .send()
        .await
        .context("failed to request NJU auth login page")?
        .error_for_status()
        .context("NJU auth login page returned an error status")?
        .text()
        .await
        .context("failed to read NJU auth login page")?;

    if login_page.contains("pwdFromId") {
        return Err(anyhow!(
            "not logged in or the login has expired; run `nju-cli login --username USERNAME --password PASSWORD` first"
        ));
    }

    Ok(())
}

fn save_castgc(castgc: String) -> Result<()> {
    let path = auth_cache_file()?;
    let updated_at_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock is earlier than UNIX epoch")?
        .as_secs();
    let cached = CachedAuth {
        castgc,
        updated_at_unix,
    };
    let json = serde_json::to_string_pretty(&cached).context("failed to serialize cached login")?;

    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_castgc() -> Result<String> {
    let path = auth_cache_file()?;
    let json = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read cached login from {}; run `nju-cli login --username USERNAME --password PASSWORD` first",
            path.display()
        )
    })?;
    let cached: CachedAuth = serde_json::from_str(&json)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(cached.castgc)
}

fn auth_cache_file() -> Result<PathBuf> {
    Ok(auth_cache_dir()?.join("auth.json"))
}

fn vpn_pending_file() -> Result<PathBuf> {
    Ok(auth_cache_dir()?.join("vpn-login.json"))
}

fn vpn_session_file() -> Result<PathBuf> {
    Ok(auth_cache_dir()?.join("vpn-session.json"))
}

fn save_vpn_operation(path: &Path, operation: &LoginOperation) -> Result<()> {
    let json = serde_json::to_string_pretty(operation)
        .context("failed to serialize Web VPN login operation")?;
    std::fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn load_vpn_operation(path: &Path) -> Result<LoginOperation> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn auth_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("auth");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
