use anyhow::{Result, ensure};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct JsonResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T> JsonResponse<T> {
    pub fn check_code(self, action: &str) -> Result<Self> {
        ensure!(self.code == 0, "{action}: {}", self.message);
        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthConfigData {
    pub first_auth: Vec<String>,
    pub security: AuthConfigSecurity,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthConfigSecurity {
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthCheckData {
    pub next_service_list: Vec<AuthService>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthService {
    pub auth_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ShortcutData {
    pub ticket: String,
}
