use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthConfigData {
    #[serde(rename = "firstAuth")]
    pub first_auth: Vec<String>,
}
