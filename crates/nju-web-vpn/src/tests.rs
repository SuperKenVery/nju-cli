use std::sync::Arc;

use anyhow::Result;
use reqwest::Url;
use reqwest_cookie_store::CookieStoreMutex;

use crate::{
    LoginOperation,
    operation::{build_client, lock_cookie_store},
    utils::{generate_device_id, web_vpn_url},
};

#[test]
fn encodes_target_host_for_atrust() -> Result<()> {
    let target = Url::parse("https://foo-bar.example.nju.edu.cn/a?q=1")?;
    assert_eq!(
        web_vpn_url(&target)?.as_str(),
        "https://foo--bar-example-nju-edu-cn.atrust.nju.edu.cn/a?q=1"
    );
    Ok(())
}

#[test]
fn serde_round_trip_keeps_session_cookies() -> Result<()> {
    let cookie_store = Arc::new(CookieStoreMutex::default());
    {
        let mut store = lock_cookie_store(&cookie_store)?;
        store.parse(
            "sid=test-session; Path=/; Secure; HttpOnly",
            &Url::parse("https://ztna.nju.edu.cn")?,
        )?;
    }
    let operation = LoginOperation {
        client: build_client(Arc::clone(&cookie_store))?,
        cookie_store,
        web_vpn_url: Url::parse("https://example-nju-edu-cn.atrust.nju.edu.cn")?,
        sid: None,
        csrf_token: "csrf".to_string(),
        auth_id: "auth-id".to_string(),
    };

    let checkpoint = serde_json::to_vec(&operation)?;
    let restored: LoginOperation = serde_json::from_slice(&checkpoint)?;
    assert_eq!(restored.csrf_token, "csrf");
    assert_eq!(restored.auth_id, "auth-id");
    let store = lock_cookie_store(&restored.cookie_store)?;
    assert_eq!(
        store
            .get("ztna.nju.edu.cn", "/", "sid")
            .map(|cookie| cookie.value()),
        Some("test-session")
    );
    Ok(())
}

#[test]
fn device_ids_match_the_browser_shape() {
    for _ in 0..100 {
        let device_id = generate_device_id();
        assert_eq!(device_id.len(), 67);
        assert!(device_id.starts_with("00"));
        assert_ne!(device_id.as_bytes()[2], b'0');
        assert!(device_id.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
