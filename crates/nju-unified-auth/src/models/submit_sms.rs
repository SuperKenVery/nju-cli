use serde::Deserialize;

#[derive(Deserialize)]
pub struct SMSResponseData {
    pub time: String,
}
