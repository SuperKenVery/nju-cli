# nju-web-vpn

Clientless access to Nanjing University Web VPN resources.

The login flow is represented by a serializable `LoginOperation`. Starting an operation sends the
SMS verification code. The caller can then serialize the operation, exit, deserialize it in
another process, and submit the code without blocking for terminal input.

```toml
[dependencies]
nju-web-vpn = "0.1"
reqwest-middleware = "0.4"
serde_json = "1"
```

```rust,no_run
use anyhow::Result;
use nju_web_vpn::{LoginOperation, WebVpnMiddleware};
use reqwest::Url;
use reqwest_middleware::ClientBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let target = Url::parse("https://example.nju.edu.cn")?;
    let operation = LoginOperation::start(target, "CASTGC cookie value").await?;
    std::fs::write(
        "web-vpn-login.json",
        serde_json::to_vec_pretty(&operation)?,
    )?;

    // A later invocation receives the SMS code from its caller.
    let mut operation: LoginOperation =
        serde_json::from_slice(&std::fs::read("web-vpn-login.json")?)?;
    operation.submit_sms("123456").await?;

    let client = ClientBuilder::new(operation.client)
        .with(WebVpnMiddleware)
        .build();
    let response = client.get(target).send().await?;
    println!("{}", response.status());

    Ok(())
}
```

Use [`nju-unified-auth`](https://crates.io/crates/nju-unified-auth) to obtain a `CASTGC` cookie.

## License

MIT
