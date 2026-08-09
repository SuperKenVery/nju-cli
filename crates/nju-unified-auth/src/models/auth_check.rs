use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthCheckData {
    #[serde(rename = "nextServiceList")]
    pub next_service_list: Vec<NextService>,
}

#[derive(Deserialize, Debug)]
pub struct NextService {
    #[serde(rename = "authId")]
    pub auth_id: String,
    #[serde(rename = "authType")]
    pub auth_type: String,
}
