use anyhow::{Context, Result};
use reqwest::{RequestBuilder, Response};

pub(crate) async fn send(request: RequestBuilder, action: &str) -> Result<Response> {
    request
        .send()
        .await
        .with_context(|| format!("failed to {action}"))?
        .error_for_status()
        .with_context(|| format!("{action} returned an error status"))
}
