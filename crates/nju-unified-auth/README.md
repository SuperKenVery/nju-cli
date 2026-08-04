# nju-unified-auth

Login helpers for Nanjing University's unified authentication service.

The crate manages the login session internally and returns the resulting `CASTGC` cookie.

```toml
[dependencies]
nju-unified-auth = "0.1"
```

```rust,no_run
use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let castgc = nju_unified_auth::login("username", "password").await?;
    println!("CASTGC={castgc}");
    Ok(())
}
```

The current authentication flow solves the slider CAPTCHA with the pure-Rust `ddddocr-tract`
matcher, generates a human-like movement track, and retries with a fresh challenge when matching is
rejected. Slider matching does not download an OCR model.

## License

MIT
