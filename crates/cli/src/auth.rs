use std::{path::PathBuf, sync::Arc, time::SystemTime};

use anyhow::{Context, Result, anyhow};
use clap::Args;
use platform_dirs::AppDirs;
use reqwest::{Url, cookie::Jar};
use serde::{Deserialize, Serialize};

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
            let client = reqwest::Client::builder()
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .context("failed to build NJU auth login client")?;

            nju_unified_auth::login(&client, username, password)
                .await
                .context("failed to login NJU unified auth")?
        }
    };

    save_castgc(castgc)?;
    println!("login saved");

    Ok(())
}

/// 构建带缓存统一认证登陆态的 client，并检查登陆态仍然有效。
pub async fn authenticated_client() -> Result<reqwest::Client> {
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

    nju_unified_auth::ensure_logged_in(&client).await?;

    Ok(client)
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

fn auth_cache_dir() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .ok_or_else(|| anyhow!("failed to resolve application cache directory"))?;
    let dir = app_dirs.cache_dir.join("auth");

    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    Ok(dir)
}
