use std::{
    collections::HashMap,
    sync::{Arc, MutexGuard},
};

use anyhow::{Context, Result, anyhow, ensure};
use reqwest::{Client, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    ser::{Error as _, SerializeStruct},
};
use serde_json::{Value, json};
use tracing::debug;

use crate::{
    models::{AuthCheckData, AuthConfigData, JsonResponse, ShortcutData},
    request::send,
    utils::{generate_device_id, web_vpn_url},
};

const AUTH_CONFIG_URL: &str = "https://ztna.nju.edu.cn/passport/v1/public/authConfig?clientType=SDPBrowserClient&platform=Mac&lang=en-US&mod=1";
const AUTH_CHECK_URL: &str = "https://ztna.nju.edu.cn/passport/v1/auth/authCheck?clientType=SDPBrowserClient&platform=Mac&lang=en-US";
const CHECK_SMS_URL: &str = "https://ztna.nju.edu.cn/passport/v1/auth/sms?action=checkcode&clientType=SDPBrowserClient&platform=Mac&lang=en-US";
const REPORT_ENV_URL: &str = "https://ztna.nju.edu.cn/controller/v1/public/reportEnv?clientType=SDPBrowserClient&platform=Mac&lang=en-US";
const AUTH_SERVER_URL: &str = "https://authserver.nju.edu.cn";
const ZTNA_HOST: &str = "ztna.nju.edu.cn";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0 Safari/537.36";

/// 一次可暂停、保存并跨进程恢复的南京大学 Web VPN 登录操作。
///
/// [`start`](Self::start) 会完成统一认证跳转并发送短信验证码。调用方随后可以通过
/// [`Serialize`] 保存状态并退出；取得验证码后通过 [`Deserialize`] 恢复操作，再使用
/// [`submit_sms`](Self::submit_sms) 继续登录。
#[derive(Debug)]
pub struct LoginOperation {
    /// 携带当前 Web VPN 会话的 HTTP client。
    pub client: Client,
    /// Web VPN 代理后的目标 URL。
    pub web_vpn_url: Url,
    /// 短信验证成功后获得的 Web VPN session ID。
    pub sid: Option<String>,
    pub(crate) cookie_store: Arc<CookieStoreMutex>,
    pub(crate) csrf_token: String,
    pub(crate) auth_id: String,
}

impl LoginOperation {
    /// 开始 Web VPN 登录并发送短信验证码。
    pub async fn start(target_url: Url, castgc: impl AsRef<str>) -> Result<Self> {
        let web_vpn_url = web_vpn_url(&target_url)?;
        let cookie_store = Arc::new(CookieStoreMutex::default());

        {
            let mut store = lock_cookie_store(&cookie_store)?;
            store
                .parse(
                    &format!(
                        "CASTGC={}; Path=/authserver; Secure; HttpOnly",
                        castgc.as_ref()
                    ),
                    &Url::parse(AUTH_SERVER_URL)?,
                )
                .context("failed to insert CASTGC into the Web VPN session")?;
        }

        let client = build_client(Arc::clone(&cookie_store))?;
        prepare_web_vpn(&client, &web_vpn_url).await?;
        let ticket = authenticate_with_unified_auth(&client).await?;
        let auth_config = get_auth_config(&client, "prepare Web VPN SMS verification").await?;
        let csrf_token = auth_config.data.security.csrf_token;
        let device_id = generate_device_id();

        report_browser_environment(&client, &ticket, &csrf_token, &device_id).await?;
        let auth_id = get_sms_auth_id(&client).await?;
        send_sms(&client, &auth_id).await?;

        Ok(Self {
            client,
            cookie_store,
            web_vpn_url,
            csrf_token,
            auth_id,
            sid: None,
        })
    }

    /// 提交短信验证码并完成登录。
    pub async fn submit_sms(&mut self, code: impl AsRef<str>) -> Result<()> {
        ensure!(self.sid.is_none(), "Web VPN login is already complete");
        let code = code.as_ref().trim();
        ensure!(!code.is_empty(), "SMS verification code must not be empty");

        send(
            self.client
                .post(CHECK_SMS_URL)
                .header("x-csrf-token", &self.csrf_token)
                .json(&json!({
                    "authId": self.auth_id,
                    "code": code,
                    "isPrevEffect": false,
                    "skipSecondaryAuth": "0",
                    "taskId": ""
                })),
            "submit the Web VPN SMS code",
        )
        .await?
        .json::<JsonResponse<Value>>()
        .await
        .context("failed to parse the Web VPN SMS verification response")?
        .check_code("Web VPN rejected the SMS verification code")?;

        self.sid = Some(
            self.current_sid()?
                .context("Web VPN login succeeded without a sid cookie")?,
        );
        Ok(())
    }

    fn current_sid(&self) -> Result<Option<String>> {
        let store = lock_cookie_store(&self.cookie_store)?;
        Ok(store
            .get(ZTNA_HOST, "/", "sid")
            .map(|cookie| cookie.value().to_string()))
    }
}

impl Serialize for LoginOperation {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let store = lock_cookie_store(&self.cookie_store).map_err(S::Error::custom)?;
        let cookies = store.iter_any().collect::<Vec<_>>();
        let mut state = serializer.serialize_struct("LoginOperation", 5)?;
        state.serialize_field("web_vpn_url", self.web_vpn_url.as_str())?;
        state.serialize_field("csrf_token", &self.csrf_token)?;
        state.serialize_field("auth_id", &self.auth_id)?;
        state.serialize_field("sid", &self.sid)?;
        state.serialize_field("cookies", &cookies)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for LoginOperation {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Fields {
            web_vpn_url: String,
            csrf_token: String,
            auth_id: String,
            sid: Option<String>,
            cookies: Vec<cookie_store::Cookie<'static>>,
        }

        let data = Fields::deserialize(deserializer)?;
        let store =
            CookieStore::from_cookies(data.cookies.into_iter().map(Ok::<_, D::Error>), true)?;
        let cookie_store = Arc::new(CookieStoreMutex::new(store));
        let client = build_client(Arc::clone(&cookie_store)).map_err(serde::de::Error::custom)?;
        let web_vpn_url = Url::parse(&data.web_vpn_url).map_err(serde::de::Error::custom)?;

        Ok(Self {
            client,
            cookie_store,
            web_vpn_url,
            csrf_token: data.csrf_token,
            auth_id: data.auth_id,
            sid: data.sid,
        })
    }
}

pub(crate) fn build_client(cookie_store: Arc<CookieStoreMutex>) -> Result<Client> {
    Client::builder()
        .cookie_provider(cookie_store)
        .user_agent(USER_AGENT)
        .build()
        .context("failed to build Web VPN HTTP client")
}

pub(crate) fn lock_cookie_store(
    cookie_store: &Arc<CookieStoreMutex>,
) -> Result<MutexGuard<'_, CookieStore>> {
    cookie_store
        .lock()
        .map_err(|_| anyhow!("Web VPN cookie store lock is poisoned"))
}

async fn prepare_web_vpn(client: &Client, web_vpn_url: &Url) -> Result<()> {
    debug!(%web_vpn_url, "requesting Web VPN target");
    let response = send(client.get(web_vpn_url.clone()), "open the Web VPN target").await?;
    ensure!(
        response.url().host_str() == Some(ZTNA_HOST)
            && response.url().path().starts_with("/portal/shortcut.html"),
        "Web VPN target redirected to an unexpected URL: {}",
        response.url()
    );
    Ok(())
}

async fn authenticate_with_unified_auth(client: &Client) -> Result<String> {
    debug!("authenticating Web VPN through NJU unified auth");
    let config = get_auth_config(client, "get Web VPN authentication config").await?;
    let authserver_url = config
        .data
        .first_auth
        .first()
        .context("Web VPN authentication config contains no unified auth URL")?;
    let mut response = send(
        client.get(authserver_url),
        "authenticate Web VPN through NJU unified auth",
    )
    .await?;

    if response.url().host_str() == Some("authserver.nju.edu.cn") {
        let mut params = HashMap::new();
        params.insert("scope", "user_profile");
        response = send(
            client.post(response.url().clone()).form(&params),
            "authorize Web VPN in NJU unified auth",
        )
        .await?;
    }

    ensure!(
        response.url().host_str() == Some(ZTNA_HOST)
            && response.url().path().starts_with("/portal/shortcut.html"),
        "NJU unified auth redirected to an unexpected URL: {}",
        response.url()
    );
    let data = response
        .url()
        .query_pairs()
        .find_map(|(name, value)| (name == "data").then(|| value.into_owned()))
        .context("Web VPN shortcut URL contains no data parameter")?;
    let shortcut: ShortcutData =
        serde_json::from_str(&data).context("failed to parse Web VPN shortcut data")?;
    Ok(shortcut.ticket)
}

async fn get_auth_config(client: &Client, action: &str) -> Result<JsonResponse<AuthConfigData>> {
    send(client.get(AUTH_CONFIG_URL), action)
        .await?
        .json::<JsonResponse<AuthConfigData>>()
        .await
        .with_context(|| format!("failed to parse response while trying to {action}"))?
        .check_code(action)
}

async fn report_browser_environment(
    client: &Client,
    ticket: &str,
    csrf_token: &str,
    device_id: &str,
) -> Result<()> {
    let body = json!({
        "ticket": ticket,
        "deviceId": device_id,
        "env": {
            "endpoint": {
                "device_id": device_id,
                "device": { "type": "browser" }
            }
        }
    });

    send(
        client
            .post(REPORT_ENV_URL)
            .header("x-csrf-token", csrf_token)
            .json(&body),
        "report the browser environment to Web VPN",
    )
    .await?
    .json::<JsonResponse<Value>>()
    .await
    .context("failed to parse the Web VPN environment response")?
    .check_code("Web VPN rejected the browser environment")?;
    Ok(())
}

async fn get_sms_auth_id(client: &Client) -> Result<String> {
    let response = send(
        client.get(AUTH_CHECK_URL),
        "prepare Web VPN SMS verification",
    )
    .await?
    .json::<JsonResponse<AuthCheckData>>()
    .await
    .context("failed to parse the Web VPN authentication check")?
    .check_code("Web VPN authentication check failed")?;

    response
        .data
        .next_service_list
        .first()
        .map(|service| service.auth_id.clone())
        .context("Web VPN offered no SMS authentication service")
}

async fn send_sms(client: &Client, auth_id: &str) -> Result<()> {
    debug!("sending Web VPN SMS verification code");
    let url = Url::parse_with_params(
        "https://ztna.nju.edu.cn/passport/v1/auth/sms",
        &[
            ("action", "sendsms"),
            ("clientType", "SDPBrowserClient"),
            ("platform", "Mac"),
            ("lang", "en-US"),
            ("isPrevEffect", "0"),
            ("taskId", ""),
            ("authId", auth_id),
        ],
    )?;
    send(client.get(url), "send the Web VPN SMS code")
        .await?
        .json::<JsonResponse<Value>>()
        .await
        .context("failed to parse the Web VPN SMS response")?
        .check_code("Web VPN failed to send the SMS code")?;
    Ok(())
}
