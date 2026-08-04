use aes::{
    Aes128,
    cipher::{Block, BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit, block_padding::Pkcs7},
};
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

const ORDER_PIN_KEY: &str = "c1h2i5n6g2o2k4a7";
const ORDER_PIN_IV: &str = "C2H3I4N5G2O3K1E4";

pub fn captcha_verification(
    token: &str,
    secret_key: Option<&str>,
    points_json: &str,
) -> Result<String> {
    let payload = format!("{token}---{points_json}");
    match secret_key {
        Some(secret_key) => encrypt_captcha_payload(&payload, secret_key),
        None => Ok(payload),
    }
}

pub fn order_pin(client_x: i64, client_y: i64) -> Result<String> {
    encrypt_order_pin(&format!("{client_x},{client_y}"))
}

pub(crate) fn encrypt_captcha_payload(payload: &str, secret_key: &str) -> Result<String> {
    encrypt_ecb_base64(payload, secret_key)
}

fn encrypt_order_pin(payload: &str) -> Result<String> {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;

    let cipher = Aes128CbcEnc::new(
        ORDER_PIN_KEY.as_bytes().into(),
        ORDER_PIN_IV.as_bytes().into(),
    );
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(payload.as_bytes());

    Ok(hex_encode(&ciphertext))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn encrypt_ecb_base64(payload: &str, key: &str) -> Result<String> {
    if key.len() != 16 {
        return Err(anyhow!("AES key must be 16 bytes"));
    }

    let cipher = Aes128::new_from_slice(key.as_bytes())
        .map_err(|_| anyhow!("failed to create AES cipher"))?;
    let mut ciphertext = payload.as_bytes().to_vec();
    let padding_len = 16 - ciphertext.len() % 16;
    ciphertext.extend(std::iter::repeat_n(padding_len as u8, padding_len));
    for chunk in ciphertext.chunks_exact_mut(16) {
        cipher.encrypt_block(Block::<Aes128>::from_mut_slice(chunk));
    }

    Ok(general_purpose::STANDARD.encode(ciphertext))
}
