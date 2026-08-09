use std::{collections::HashMap, io};

use anyhow::{Context, Result, ensure};
use reqwest::{Client, Url};
use serde_json::json;
use tracing::debug;

use crate::{
    login::{build_logged_in_client, build_login_client_with_cookie_store},
    models::{
        JsonResponse, auth_check::AuthCheckData, auth_config::AuthConfigData,
        send_sms::SendSMSResponseData, shortcut_html_data, submit_sms::SMSResponseData,
    },
    request, utils,
};

const AUTH_CONFIG_URL: &str = "https://ztna.nju.edu.cn/passport/v1/public/authConfig?clientType=SDPBrowserClient&platform=Mac&lang=en-US&mod=1";
const REPORT_ENV_URL: &str = "https://ztna.nju.edu.cn/controller/v1/public/reportEnv?clientType=SDPBrowserClient&platform=Mac&lang=en-US";

pub async fn login_to_vpn(
    target_url: &Url,
    username: impl Into<String>,
    password: impl AsRef<str>,
    castgc: Option<String>,
) -> Result<String> {
    let (client, cookie_store) = match castgc {
        None => {
            debug!("Logging in to authserver");
            build_logged_in_client(username, password).await?
        }
        Some(cookie) => {
            let (client, cookie_store) = build_login_client_with_cookie_store()?;

            {
                let mut store = cookie_store.lock().unwrap();
                store
                    .parse(
                        &format!("CASTGC={cookie}; Path=/authserver; Secure; HttpOnly"),
                        &Url::parse("https://authserver.nju.edu.cn")?,
                    )
                    .context("failed to insert CASTGC into cookie store")?;
            }

            (client, cookie_store)
        }
    };
    // debug!("Logging in to authserver");
    // let (client, cookie_store) = build_logged_in_client(username, password).await?;

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
    let auth_config = client
        .get(AUTH_CONFIG_URL)
        .send()
        .await?
        .json::<JsonResponse<AuthConfigData>>()
        .await?
        .check_code("Failed to get auth config from ztna")?;

    let authserver_url = auth_config
        .data
        .first_auth
        .get(0)
        .context("Failed to get authserver URL from ztna VPN")?;
    let mut authserver_login_response = client.get(authserver_url).send().await?;
    if authserver_login_response.url().host_str() == Some("authserver.nju.edu.cn") {
        // Needs clicking "Authorize"
        // Just POST at the same URL.
        let mut params = HashMap::new();
        params.insert("scope", "user_profile");

        authserver_login_response = client
            .post(authserver_login_response.url().clone())
            .form(&params)
            .send()
            .await?;
    } // Otherwise, it should be at https://ztna.nju.edu.cn/portal/shortcut.html?xxxxx
    ensure!(
        authserver_login_response
            .url()
            .as_str()
            .starts_with("https://ztna.nju.edu.cn/portal/shortcut.html"),
        "Failed to login ztna via authserver"
    );
    let (_, shortcut_data_str) = authserver_login_response
        .url()
        .query_pairs()
        .into_iter()
        .find(|(name, _value)| name == "data")
        .context("Failed to find data in URL query of shortcut.html")?;
    let shortcut_data: shortcut_html_data::Data =
        serde_json::from_str(&shortcut_data_str.to_string())?;
    let ticket = shortcut_data.ticket;

    debug!("Preparing SMS verification");
    let auth_config = client
        .get(AUTH_CONFIG_URL)
        .send()
        .await?
        .json::<JsonResponse<AuthConfigData>>()
        .await?
        .check_code("Failed to get auth config from ztna for sending SMS")?;
    let csrf_token = &auth_config.data.security.csrf_token;

    let device_id = utils::get_or_create_vpn_device_id()?;
    report_browser_environment(&client, &ticket, csrf_token, &device_id).await?;

    let auth_check = client
        .get("https://ztna.nju.edu.cn/passport/v1/auth/authCheck?clientType=SDPBrowserClient&platform=Mac&lang=en-US")
        .send().await?
        .json::<JsonResponse<AuthCheckData>>().await?.check_code("Failed to get auth_id when trying to send SMS code")?;

    debug!("Sending SMS code");
    let auth_id = auth_check
        .data
        .next_service_list
        .get(0)
        .context("Failed to get authId from authCheck data")?
        .auth_id
        .as_str();
    let _send_sms = client
        .get(format!("https://ztna.nju.edu.cn/passport/v1/auth/sms?action=sendsms&clientType=SDPBrowserClient&platform=Mac&lang=en-US&isPrevEffect=0&taskId=&authId={}", auth_id))
        .send().await?
        .json::<JsonResponse<SendSMSResponseData>>().await?.check_code("Failed to send SMS code")?;

    // submit sms code: https://ztna.nju.edu.cn/passport/v1/auth/sms?action=checkcode&clientType=SDPBrowserClient&platform=Mac&lang=en-US
    let mut sms_code = String::new();
    println!("Please input the SMS code below:");
    io::stdin().read_line(&mut sms_code)?;
    let _submit_sms: JsonResponse<SMSResponseData> = client.post("https://ztna.nju.edu.cn/passport/v1/auth/sms?action=checkcode&clientType=SDPBrowserClient&platform=Mac&lang=en-US")
        .json(&json!({
            "authId": auth_id,
            "code": sms_code,
            "isPrevEffect": false,
            "skipSecondaryAuth": "0",
            "taskId": ""
        }))
        .header("x-csrf-token", csrf_token)
        .send().await?.json().await?;

    let cookie_store = cookie_store.lock().expect("Failed to lock cookie store");
    cookie_store
        .get("ztna.nju.edu.cn", "/", "sid")
        .map(|cookie| cookie.value().to_string())
        .context("sid not found")
}

async fn report_browser_environment(
    client: &Client,
    ticket: &str,
    csrf_token: &str,
    device_id: &str,
) -> Result<()> {
    // let trace_id = utils::generate_sdp_trace_id();
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

    request::send(
        client
            .post(REPORT_ENV_URL)
            .header("x-csrf-token", csrf_token)
            // .header("x-sdp-traceid", trace_id)
            .json(&body),
        "report browser environment to ztna",
    )
    .await?
    .json::<JsonResponse<()>>()
    .await
    .context("failed to parse ztna reportEnv response")?
    .check_code("ztna rejected the browser environment report")?;

    Ok(())
}
