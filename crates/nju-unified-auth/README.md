# nju-unified-auth

Login helpers for Nanjing University's unified authentication service.

The crate accepts a caller-owned `reqwest::Client`, allowing applications to keep the same cookie
store across unified authentication and subsequent service redirects.

```toml
[dependencies]
nju-unified-auth = "0.1"
reqwest = { version = "0.12", features = ["cookies"] }
```

```rust,no_run
use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let castgc = nju_unified_auth::login(&client, "username", "password").await?;
    println!("CASTGC={castgc}");
    Ok(())
}
```

The built-in CAPTCHA recognizer uses the pure-Rust `ddddocr-tract` backend. On first use it
downloads a revision-pinned model from GitHub, verifies its SHA-256 checksum, and caches it in the
platform cache directory. Download failures explicitly report that the ddddocr model could not be
downloaded from GitHub and retain the underlying network error. Set `DDDDOCR_MODEL_DIR` to override
the model cache directory.

## License

MIT
