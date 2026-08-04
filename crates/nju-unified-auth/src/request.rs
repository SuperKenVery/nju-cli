//! Simple wrapper around reqwest.

use anyhow::{Context, Result};
use reqwest::{RequestBuilder, Response};

/// Send request, returning error for bad status
pub(crate) async fn send(request: RequestBuilder, action: &str) -> Result<Response> {
    request
        .send()
        .await
        .with_context(|| format!("failed to {action}"))?
        .error_for_status()
        .with_context(|| format!("{action} returned an error status"))
}

/// Send request and get text, returning error for bad status
pub(crate) async fn text(request: RequestBuilder, action: &str) -> Result<String> {
    send(request, action)
        .await?
        .text()
        .await
        .with_context(|| format!("failed to read response after trying to {action}"))
}
