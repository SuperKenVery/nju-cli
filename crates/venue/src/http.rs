use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::model::ApiResponse;

const API_BASE_URL: &str = "https://ggtypt.nju.edu.cn/venue-server";
const SSO_LOGIN_URL: &str = "https://authserver.nju.edu.cn/authserver/login?service=https://ggtypt.nju.edu.cn/venue-server/sso/manageLogin";
const APP_KEY: &str = "8fceb735082b5a529312040b58ea780b";
const SIGN_SECRET: &str = "c640ca392cd45fb3a55b00a63a86c618";

pub struct VenueSession<'a> {
    pub(crate) client: &'a common::Client,
    pub(crate) authorization: Option<String>,
}

/// 通过统一认证建立体育场馆业务站点 session。
pub async fn prepare_session(client: &common::Client) -> Result<VenueSession<'_>> {
    let response = client
        .get(SSO_LOGIN_URL)
        .send()
        .await
        .context("failed to open venue SSO login URL")?
        .error_for_status()
        .context("venue SSO login returned an error status")?;
    let oauth_token = response
        .url()
        .query_pairs()
        .find(|(key, _)| key == "oauth_token")
        .map(|(_, value)| value.into_owned());

    let Some(oauth_token) = oauth_token else {
        return Ok(VenueSession {
            client,
            authorization: None,
        });
    };

    let empty = std::collections::BTreeMap::<String, String>::new();
    let login = post_form_api_raw(
        client,
        None,
        "/api/login",
        &empty,
        &[("oauth-token", oauth_token.as_str())],
    )
    .await
    .context("failed to exchange venue oauth token")?;
    let authorization = login
        .get("token")
        .and_then(|token| token.get("access_token"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("venue login response did not include access token"))?
        .to_string();
    let role_id = login
        .get("roles")
        .and_then(Value::as_array)
        .and_then(|roles| roles.first())
        .and_then(|role| role.get("id"))
        .cloned();

    let authorization = if let Some(role_id) = role_id {
        let mut form = std::collections::BTreeMap::new();
        form.insert("roleid".to_string(), value_to_form_string(&role_id));
        let role_login = post_form_api_raw(
            client,
            Some(&authorization),
            "/roleLogin",
            &form,
            &[("cgAuthorization", authorization.as_str())],
        )
        .await
        .context("failed to select venue role")?;
        role_login
            .get("token")
            .and_then(|token| token.get("access_token"))
            .and_then(Value::as_str)
            .unwrap_or(&authorization)
            .to_string()
    } else {
        authorization
    };

    Ok(VenueSession {
        client,
        authorization: Some(authorization),
    })
}

pub(crate) async fn get_api(
    session: &VenueSession<'_>,
    path: &str,
    query: &[(&str, String)],
) -> Result<Value> {
    get_api_typed(session, path, query).await
}

pub(crate) async fn get_api_typed<T: DeserializeOwned + Default>(
    session: &VenueSession<'_>,
    path: &str,
    query: &[(&str, String)],
) -> Result<T> {
    let mut query = query.to_vec();
    let timestamp = nocache_timestamp()?;
    query.push(("nocache", timestamp.clone()));
    let sign = sign(path, &query, &timestamp);
    let mut request = session
        .client
        .get(api_url(path))
        .header("app-key", APP_KEY)
        .header("timestamp", &timestamp)
        .header("sign", sign)
        .query(&query);
    if let Some(authorization) = &session.authorization {
        request = request.header("cgAuthorization", authorization);
    }
    let response = request
        .send()
        .await
        .with_context(|| format!("failed to GET {path}"))?
        .error_for_status()
        .with_context(|| format!("GET {path} returned an error status"))?;
    let response = decode_api_response(response, "GET", path).await?;

    into_data(response)
}

pub(crate) async fn post_form_api<T: Serialize + ?Sized>(
    session: &VenueSession<'_>,
    path: &str,
    form: &T,
) -> Result<Value> {
    post_form_api_typed(session, path, form).await
}

pub(crate) async fn post_form_api_typed<T: Serialize + ?Sized, R: DeserializeOwned + Default>(
    session: &VenueSession<'_>,
    path: &str,
    form: &T,
) -> Result<R> {
    post_form_api_raw_typed(
        session.client,
        session.authorization.as_deref(),
        path,
        form,
        &[],
    )
    .await
}

async fn post_form_api_raw<T: Serialize + ?Sized>(
    client: &common::Client,
    authorization: Option<&str>,
    path: &str,
    form: &T,
    extra_headers: &[(&str, &str)],
) -> Result<Value> {
    post_form_api_raw_typed(client, authorization, path, form, extra_headers).await
}

async fn post_form_api_raw_typed<T: Serialize + ?Sized, R: DeserializeOwned + Default>(
    client: &common::Client,
    authorization: Option<&str>,
    path: &str,
    form: &T,
    extra_headers: &[(&str, &str)],
) -> Result<R> {
    let timestamp = nocache_timestamp()?;
    let form_value = serde_json::to_value(form).context("failed to convert form to JSON")?;
    let sign_params = sign_params_from_value(&form_value)?;
    let sign = sign(path, &sign_params, &timestamp);
    let mut request = client
        .post(api_url(path))
        .header("app-key", APP_KEY)
        .header("timestamp", &timestamp)
        .header("sign", sign)
        .form(form);
    if let Some(authorization) = authorization {
        request = request.header("cgAuthorization", authorization);
    }
    for (key, value) in extra_headers {
        request = request.header(*key, *value);
    }
    let response = request
        .send()
        .await
        .with_context(|| format!("failed to POST {path}"))?
        .error_for_status()
        .with_context(|| format!("POST {path} returned an error status"))?;
    let response = decode_api_response(response, "POST", path).await?;

    into_data(response)
}

async fn decode_api_response<T: DeserializeOwned>(
    response: reqwest::Response,
    method: &str,
    path: &str,
) -> Result<ApiResponse<T>> {
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = response
        .text()
        .await
        .with_context(|| format!("failed to read {method} {path} response body"))?;

    if body.trim().is_empty() {
        return Err(anyhow!(
            "{method} {path} returned an empty response body; status={status}, content-type={content_type}"
        ));
    }

    serde_json::from_str(&body).with_context(|| {
        let snippet = body.chars().take(500).collect::<String>();
        format!(
            "failed to decode {method} {path} response; status={status}, content-type={content_type}, body={snippet:?}"
        )
    })
}

fn into_data<T: Default>(response: ApiResponse<T>) -> Result<T> {
    if response.code == 200 {
        Ok(response.data.unwrap_or_default())
    } else {
        let message = response.message.or(response.message_en).unwrap_or_default();
        Err(anyhow!(
            "venue API returned code {}: {}",
            response.code,
            message
        ))
    }
}

fn api_url(path: &str) -> String {
    format!("{API_BASE_URL}{path}")
}

fn nocache_timestamp() -> Result<String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is earlier than UNIX epoch")?
        .as_millis()
        .to_string())
}

fn sign(path: &str, params: &[(&str, String)], timestamp: &str) -> String {
    let mut sorted = params
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .collect::<Vec<_>>();
    sorted.sort_by_key(|(key, _)| *key);

    let mut payload = String::from(SIGN_SECRET);
    payload.push_str(path);
    for (key, value) in sorted {
        payload.push_str(key);
        payload.push_str(value);
    }
    payload.push_str(timestamp);
    payload.push(' ');
    payload.push_str(SIGN_SECRET);

    format!("{:x}", md5::compute(payload))
}

fn sign_params_from_value(value: &Value) -> Result<Vec<(&str, String)>> {
    let mut params = Vec::new();

    if let Some(object) = value.as_object() {
        for (key, value) in object {
            push_sign_param(&mut params, key, value);
        }
        return Ok(params);
    }

    if let Some(items) = value.as_array() {
        for item in items {
            let Some(pair) = item.as_array() else {
                continue;
            };
            if pair.len() != 2 {
                continue;
            }
            let Some(key) = pair.first().and_then(Value::as_str) else {
                continue;
            };
            if let Some(value) = pair.get(1) {
                push_sign_param(&mut params, key, value);
            }
        }
        return Ok(params);
    }

    Err(anyhow!(
        "form data must serialize to an object or key-value array"
    ))
}

fn push_sign_param<'a>(params: &mut Vec<(&'a str, String)>, key: &'a str, value: &Value) {
    match value {
        Value::Null | Value::Array(_) | Value::Object(_) => {}
        Value::String(value) => params.push((key, value.trim().to_string())),
        Value::Bool(value) => params.push((key, value.to_string())),
        Value::Number(value) => params.push((key, value.to_string())),
    }
}

fn value_to_form_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => value.to_string(),
    }
}
