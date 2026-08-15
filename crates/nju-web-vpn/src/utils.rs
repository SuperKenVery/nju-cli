use anyhow::{Context, Result};
use rand::Rng;
use reqwest::Url;

const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn web_vpn_url(target_url: &Url) -> Result<Url> {
    let target_host = target_url
        .host_str()
        .context("target URL has no host")?
        .replace('-', "--")
        .replace('.', "-");
    let mut url = target_url.clone();
    url.set_host(Some(&format!("{target_host}.atrust.nju.edu.cn")))?;
    Ok(url)
}

pub(crate) fn generate_device_id() -> String {
    let mut rng = rand::rng();
    let mut device_id = String::with_capacity(67);
    device_id.push_str("00");
    device_id.push(LOWER_HEX[rng.random_range(1..LOWER_HEX.len())] as char);
    for _ in 0..64 {
        device_id.push(LOWER_HEX[rng.random_range(0..LOWER_HEX.len())] as char);
    }
    device_id
}
