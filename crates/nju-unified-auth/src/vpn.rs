use std::{collections::HashMap, io};

use anyhow::{Context, Result, anyhow, ensure};
use reqwest::{
    Client, Response, Url,
    header::{self, HeaderMap, HeaderValue},
};
use serde_json::json;
use tracing::debug;

use crate::{
    captcha::verify_slider_captcha,
    login::build_logged_in_client,
    models::{
        JsonResponse, auth_check::AuthCheckData, auth_config::AuthConfigData,
        submit_sms::SMSResponseData,
    },
    request, utils,
};

pub async fn login_to_vpn(
    target_url: &Url,
    username: impl Into<String>,
    password: impl AsRef<str>,
) -> Result<String> {
    debug!("Logging in to authserver");
    let (client, cookie_store) = build_logged_in_client(username, password).await?;

    debug!("Requesting atrust");
    let target_subdomain = target_url
        .host_str()
        .context("No host in target URL")?
        .replace("-", "--")
        .replace(".", "-");
    let target_host = format!("{}.atrust.nju.edu.cn", target_subdomain);
    let mut vpn_url = target_url.clone();
    vpn_url.set_host(Some(&target_host))?;

    let prepare_vpn_login = client.get(vpn_url.clone()).send().await?;
    ensure!(
        prepare_vpn_login.url().host_str() == Some("ztna.nju.edu.cn")
            && prepare_vpn_login
                .url()
                .path()
                .starts_with("/portal/shortcut.html"),
        format!(
            "First request to NJU vpn failed. Requested={}, arrived at {}",
            vpn_url,
            prepare_vpn_login.url()
        )
    );

    debug!("Authenticating atrust in unified auth");
    // Equivalent of clicking "log in"
    let auth_config: JsonResponse<AuthConfigData> = client.get("https://ztna.nju.edu.cn/passport/v1/public/authConfig?clientType=SDPBrowserClient&platform=Mac&lang=en-US&mod=1").send().await?.json().await?;
    let authserver_url = auth_config
        .data
        .first_auth
        .get(0)
        .context("Failed to get authserver URL from ztna VPN")?;
    let authserver_login_response = client.get(authserver_url).send().await?;
    if authserver_login_response.url().host_str() == Some("authserver.nju.edu.cn") {
        // Needs clicking "Authorize"
        // Just POST at the same URL.
        let mut params = HashMap::new();
        params.insert("scope", "user_profile");

        client
            .post(authserver_login_response.url().clone())
            .form(&params)
            .send()
            .await?;
    } // Otherwise, it should be at https://ztna.nju.edu.cn/portal/shortcut.html?xxxxx

    debug!("Preparing SMS verification");
    let auth_check: JsonResponse<AuthCheckData> = client.get("https://ztna.nju.edu.cn/passport/v1/auth/authCheck?clientType=SDPBrowserClient&platform=Mac&lang=en-US").send().await?.json().await?;

    let auth_id = auth_check
        .data
        .next_service_list
        .get(0)
        .context("Failed to get authId from authCheck data")?
        .auth_id
        .as_str();
    let _send_sms = client.get(format!("https://ztna.nju.edu.cn/passport/v1/auth/sms?action=sendsms&clientType=SDPBrowserClient&platform=Mac&lang=en-US&isPrevEffect=0&taskId=&authId={}", auth_id)).send().await?;

    // submit sms code: https://ztna.nju.edu.cn/passport/v1/auth/sms?action=checkcode&clientType=SDPBrowserClient&platform=Mac&lang=en-US
    let mut sms_code = String::new();
    println!("Please input the SMS code below:");
    io::stdin().read_line(&mut sms_code);
    let _submit_sms: JsonResponse<SMSResponseData> = client.post("https://ztna.nju.edu.cn/passport/v1/auth/sms?action=checkcode&clientType=SDPBrowserClient&platform=Mac&lang=en-US")
        .json(&json!({
            "authId": auth_id,
            "code": sms_code,
            "isPrevEffect": false,
            "skipSecondaryAuth": 0,
            "taskId": ""
        }))
        .send().await?.json().await?;

    let cookie_store = cookie_store.lock().expect("Failed to lock cookie store");
    cookie_store
        .get("ztna.nju.edu.cn", "/", "sid")
        .map(|cookie| cookie.value().to_string())
        .context("sid not found")
}

#[cfg(test)]
mod test {
    use reqwest::Url;

    use crate::vpn::login_to_vpn;

    #[tokio::test]
    async fn test_vpn_login() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
        let sid = login_to_vpn(
            &Url::parse("https://tuanwei.nju.edu.cn").unwrap(),
            "652026330028",
            "$hqw7w7MA!cvtJ",
        )
        .await
        .unwrap();
        println!("sid: {}", sid)
    }
}
