use nju_unified_auth::login_to_vpn;
use tracing;
use anyhow::{Result, Context};
use reqwest::Url;
use std::fs::File;
use std::io::Read;

#[tokio::test]
#[ignore = "requires live NJU credentials in tests/auth.secret"]
async fn test_vpn_login() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let secret =
        std::fs::read_to_string("tests/auth.secret").context("failed to load tests/auth.secret")?;
    let username = secret_value(&secret, "NJU_USERNAME").unwrap();
    let password = secret_value(&secret, "NJU_PASSWORD").unwrap();

    let sid = login_to_vpn(
        &Url::parse("https://tuanwei.nju.edu.cn").unwrap(),
        username,
        password,
        secret_value(&secret, "CASTGC")
    ).await?;
    println!("sid: {}", sid);

    Ok(())
}

fn secret_value(secret: &str, name: &str) -> Option<String> {
    let value = secret.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == name).then(|| value.trim())
    })?;

    Some(
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .or_else(|| {
                value
                    .strip_prefix('\'')
                    .and_then(|value| value.strip_suffix('\''))
            })
            .unwrap_or(value)
            .to_string(),
    )
}
