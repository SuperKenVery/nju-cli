use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendSMSResponseData {
    pub current_service: String,
    // pub interval: String,
    pub next_service: String,
    pub next_service_list: Vec<NextService>,
    pub tips: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NextService {
    pub auth_id: String,
    pub auth_name: String,
    pub auth_type: String,
    pub description: String,
    pub sub_type: String
}
