use std::collections::HashMap;

use aes::{
    Aes128,
    cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7},
};
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use scraper::{Html, Selector};

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
}
