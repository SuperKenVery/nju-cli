use anyhow::{Context, Result};

#[tokio::test]
#[ignore = "requires live NJU credentials in tests/auth.secret"]
async fn logs_in_with_secret_credentials() -> Result<()> {
    let secret =
        std::fs::read_to_string("tests/auth.secret").context("failed to load tests/auth.secret")?;
    let username = secret_value(&secret, "NJU_USERNAME").context("NJU_USERNAME is missing")?;
    let password = secret_value(&secret, "NJU_PASSWORD").context("NJU_PASSWORD is missing")?;

    let castgc = nju_unified_auth::login(username, password).await?;

    assert!(!castgc.trim().is_empty());

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

#[test]
fn reads_secret_values_without_environment_expansion() {
    let secret = "NJU_USERNAME=user\nNJU_PASSWORD=$literal-password\n";

    assert_eq!(
        secret_value(secret, "NJU_USERNAME").as_deref(),
        Some("user")
    );
    assert_eq!(
        secret_value(secret, "NJU_PASSWORD").as_deref(),
        Some("$literal-password")
    );
}
