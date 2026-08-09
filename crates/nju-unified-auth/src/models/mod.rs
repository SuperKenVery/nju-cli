pub mod auth_check;
pub mod auth_config;
pub mod submit_sms;
pub mod send_sms;
pub mod shortcut_html_data;

use serde::Deserialize;
use anyhow::{Result, anyhow};

#[derive(Deserialize, Debug)]
pub struct JsonResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T> JsonResponse<T> {
    pub fn check_code(self, fail_msg: &str) -> Result<Self> {
        if self.code==0 {
            Ok(self)
        }else {
            Err(anyhow!("{}: {}", fail_msg, self.message))
        }
    }
}
