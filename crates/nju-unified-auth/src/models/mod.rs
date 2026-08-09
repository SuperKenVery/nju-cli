pub mod auth_check;
pub mod auth_config;
pub mod submit_sms;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct JsonResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}
