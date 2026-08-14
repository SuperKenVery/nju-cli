use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use reqwest::{
    Client, Response,
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
    let (_logged_in_client, cookie_store) = build_logged_in_client(username, password).await?;
    let cookie_store = cookie_store
        .lock()
        .map_err(|_| anyhow!("NJU auth cookie store lock is poisoned"))?;

    cookie_store
        .get("authserver.nju.edu.cn", "/authserver", "CASTGC")
        .map(|cookie| cookie.value().to_string())
        .context("CASTGC was not found in the cookie store")
}

async fn build_logged_in_client(
    username: impl Into<String>,
    password: impl AsRef<str>,
) -> Result<(Client, Arc<CookieStoreMutex>)> {
    let (client, cookie_store) = build_login_client_with_cookie_store()?;
    let login_page = request_login_page(&client).await?;
    let context = utils::extract_context(&login_page)?;

    verify_slider_captcha(&client).await?;
    submit_login(&client, context, username.into(), password.as_ref()).await?;

    Ok((client, cookie_store))
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
) -> Result<String> {
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

    let response = request::send(
        client.post(LOGIN_URL).form(&context),
        "submit NJU auth login form",
    )
    .await?;

    extract_castgc(response).await
}

async fn extract_castgc(response: Response) -> Result<String> {
    if let Some(cookie) = response.cookies().find(|cookie| cookie.name() == "CASTGC") {
        return Ok(cookie.value().to_string());
    }

    let html = response
        .text()
        .await
        .context("failed to read NJU auth login failure page")?;
    Err(anyhow!(utils::extract_login_error(&html).unwrap_or_else(
        || { "NJU auth login failed, but no error message was found".to_string() }
    )))
}
