use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use aes::{
    Aes128,
    cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7},
};
use anyhow::{Context, Result, anyhow, ensure};
use base64::{Engine as _, engine::general_purpose};
use platform_dirs::AppDirs;
use rand::Rng;
use scraper::{Html, Selector};

const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

/// 从统一认证登录页提取提交登录表单需要的字段。
pub(crate) fn extract_context(login_page: &str) -> Result<HashMap<String, String>> {
    let document = Html::parse_document(login_page);
    let selector = Selector::parse("form#pwdFromId input")
        .map_err(|err| anyhow!("failed to parse login input selector: {err}"))?;
    let mut context = HashMap::new();

    for input in document.select(&selector) {
        let value = input.value();
        let Some(name) = value.attr("id").or_else(|| value.attr("name")) else {
            continue;
        };
        let Some(input_value) = value.attr("value") else {
            continue;
        };

        context.insert(name.to_string(), input_value.to_string());
    }

    Ok(context)
}

pub(crate) fn encrypt(plaintext: &str, key: &[u8]) -> Result<String> {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;

    let key: &[u8; 16] = key.try_into().with_context(|| {
        format!(
            "NJU auth encryption key must be 16 bytes, got {}",
            key.len()
        )
    })?;
    let iv = [b'a'; 16];
    let cipher = Aes128CbcEnc::new(key.into(), (&iv).into());
    let padded_plaintext = ["a".repeat(64), plaintext.to_string()].concat();
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(padded_plaintext.as_bytes());

    Ok(general_purpose::STANDARD.encode(ciphertext))
}

pub(crate) fn safe_secure(small_image: &[u8]) -> Result<&[u8]> {
    small_image
        .get(small_image.len().saturating_sub(16)..)
        .filter(|key| key.len() == 16)
        .context("NJU auth slider captcha smallImage is shorter than 16 bytes")
}

pub(crate) fn extract_login_error(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector =
        Selector::parse("div#pwdLoginDiv span#showErrorTip, form#casLoginForm span.auth_error")
            .ok()?;

    document
        .select(&selector)
        .next()
        .map(|node| node.text().collect::<String>().trim().to_string())
        .filter(|message| !message.is_empty())
}

/// 生成与 aTrust Web 端相同形状的设备 ID。
///
/// Web 端会把 RSA 运算结果的前导零替换为两个零；服务端无法用请求中的信息
/// 还原该运算，因此这里直接生成等长的小写十六进制字符串。
pub(crate) fn generate_vpn_device_id() -> String {
    let mut rng = rand::rng();
    let mut device_id = String::with_capacity(67);
    device_id.push_str("00");
    device_id.push(LOWER_HEX[rng.random_range(1..LOWER_HEX.len())] as char);

    for _ in 0..64 {
        device_id.push(LOWER_HEX[rng.random_range(0..LOWER_HEX.len())] as char);
    }

    device_id
}

/// 生成请求头 `x-sdp-traceid` 使用的 8 位小写十六进制字符串。
pub(crate) fn generate_sdp_trace_id() -> String {
    let mut rng = rand::rng();
    (0..8)
        .map(|_| LOWER_HEX[rng.random_range(0..LOWER_HEX.len())] as char)
        .collect()
}

/// 读取已有的 aTrust Web 设备 ID；不存在时生成、保存并返回。
pub(crate) fn get_or_create_vpn_device_id() -> Result<String> {
    let path = vpn_device_id_path()?;
    get_or_create_vpn_device_id_at(&path)
}

fn get_or_create_vpn_device_id_at(path: &Path) -> Result<String> {
    if let Some(device_id) = load_vpn_device_id_from(path)? {
        return Ok(device_id);
    }

    let device_id = generate_vpn_device_id();
    save_vpn_device_id_to(path, &device_id)?;
    Ok(device_id)
}

fn save_vpn_device_id_to(path: &Path, device_id: &str) -> Result<()> {
    ensure!(
        is_valid_vpn_device_id(device_id),
        "refusing to save an invalid aTrust VPN device ID"
    );

    let parent = path
        .parent()
        .context("aTrust VPN device ID path has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(path, device_id)
        .with_context(|| format!("failed to write aTrust VPN device ID to {}", path.display()))
}

fn vpn_device_id_path() -> Result<PathBuf> {
    let app_dirs = AppDirs::new(Some("nju-cli"), true)
        .context("failed to resolve application data directory")?;
    Ok(app_dirs.data_dir.join("auth").join("vpn-device-id"))
}

fn load_vpn_device_id_from(path: &Path) -> Result<Option<String>> {
    let saved = match fs::read_to_string(path) {
        Ok(saved) => saved,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to read saved aTrust VPN device ID from {}",
                    path.display()
                )
            });
        }
    };
    let device_id = saved.trim();

    Ok(Some(device_id.to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_context_by_id_like_the_auth_page() -> Result<()> {
        let html = r#"
            <form id="pwdFromId">
                <input type="hidden" id="execution" name="ignored" value="e1s1">
                <input type="text" id="username" value="">
                <input type="hidden" name="fallback" value="value">
            </form>
        "#;

        let context = extract_context(html)?;
        assert_eq!(context.get("execution").map(String::as_str), Some("e1s1"));
        assert_eq!(context.get("username").map(String::as_str), Some(""));
        assert_eq!(context.get("fallback").map(String::as_str), Some("value"));

        Ok(())
    }

    #[test]
    fn derives_captcha_key_from_last_sixteen_bytes() -> Result<()> {
        let image = b"prefix0123456789abcdef";
        assert_eq!(safe_secure(image)?, b"0123456789abcdef");
        assert!(safe_secure(b"too short").is_err());

        Ok(())
    }

    #[test]
    fn generates_browser_shaped_vpn_identifiers() {
        for _ in 0..100 {
            assert!(is_valid_vpn_device_id(&generate_vpn_device_id()));
        }

        let trace_id = generate_sdp_trace_id();
        assert_eq!(trace_id.len(), 8);
        assert!(
            trace_id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
    }

}
