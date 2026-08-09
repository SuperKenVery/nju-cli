use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfigData {
    pub first_auth: Vec<String>,
    pub security: AuthConfigSecurity,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfigSecurity {
    pub csrf_token: String,
}
