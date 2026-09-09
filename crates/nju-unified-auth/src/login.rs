use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use reqwest::{
    Client, StatusCode,
    header::{self, HeaderMap, HeaderValue},
};
use reqwest_cookie_store::CookieStoreMutex;

use crate::{captcha::verify_slider_captcha, request, utils};

const LOGIN_URL: &str = "https://authserver.nju.edu.cn/authserver/login";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6.1 Safari/605.1.15";

/// 登录南京大学统一认证，直接返回登录成功后的 CASTGC cookie。
///
/// 登录流程会使用内部 client 复用 cookie 并禁用自动重定向，以便从登录响应中读取
/// CASTGC。滑块验证码会自动识别，并在识别失败时更换验证码重试。
pub async fn login(username: impl Into<String>, password: impl AsRef<str>) -> Result<String> {
    let (_logged_in_client, _cookie_store, castgc_cookie) =
        build_logged_in_client(username, password).await?;

    Ok(castgc_cookie)
}

/// Build a reqwest client logged in to nju authserver.
///
/// Returns: (rqwest client, cookie store, CASTGC cookie value)
async fn build_logged_in_client(
    username: impl Into<String>,
    password: impl AsRef<str>,
) -> Result<(Client, Arc<CookieStoreMutex>, String)> {
    let (client, cookie_store) = build_login_client_with_cookie_store()?;
    let login_page = request_login_page(&client).await?;
    let context = utils::extract_context(&login_page)?;

    verify_slider_captcha(&client).await?;
    let login_response = submit_login(&client, context, username.into(), password.as_ref()).await?;

    let query_castgc = {
        let unlocked_cookie_store = cookie_store
            .lock()
            .map_err(|_| anyhow!("NJU auth cookie store lock is poisoned while testing log in"))?;

        unlocked_cookie_store
            .get("authserver.nju.edu.cn", "/authserver", "CASTGC")
            .map(|cookie| cookie.value().to_string())
    };

    match query_castgc {
        Some(castgc) => Ok((client, cookie_store, castgc)),
        None => {
            let page = login_response.text().await?;
            let err = utils::extract_login_error(&page)
                .unwrap_or_else(|| format!("Failed to find err msg, raw page: {}", page));
            Err(anyhow!("Failed to login to NJU authserver: {}", err))
        }
    }
}

fn build_login_client_with_cookie_store() -> Result<(Client, Arc<CookieStoreMutex>)> {
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://authserver.nju.edu.cn"),
    );
    headers.insert(header::REFERER, HeaderValue::from_static(LOGIN_URL));

    let cookie_provider = Arc::new(CookieStoreMutex::default());
    let client = Client::builder()
        .cookie_provider(cookie_provider.clone())
        .default_headers(headers)
        .build()
        .context("failed to build NJU auth login client")?;

    Ok((client, cookie_provider))
}

async fn request_login_page(client: &Client) -> Result<String> {
    request::text(client.get(LOGIN_URL), "request NJU auth login page").await
}

async fn submit_login(
    client: &Client,
    mut context: std::collections::HashMap<String, String>,
    username: String,
    password: &str,
) -> Result<reqwest::Response> {
    let salt = context
        .remove("pwdEncryptSalt")
        .context("failed to find password encryption salt")?;

    context.insert("username".to_string(), username);
    context.insert(
        "password".to_string(),
        utils::encrypt(password, salt.as_bytes())?,
    );
    context.insert("captcha".to_string(), String::new());
    context.insert("dllt".to_string(), "mobileLogin".to_string());

    let response = client
        .post(LOGIN_URL)
        .form(&context)
        .send()
        .await
        .with_context(|| "failed to submit NJU auth login form")?;

    // 账号或密码错误时认证服务器会返回 401，直接给出明确的错误信息。
    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(anyhow!("Password is wrong"));
    }

    response
        .error_for_status()
        .context("submit NJU auth login form returned an error status")
}
