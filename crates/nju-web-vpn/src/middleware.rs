use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response, Url};
use reqwest_middleware::{Middleware, Next};

use crate::utils::web_vpn_url;

const ZTNA_HOST: &str = "ztna.nju.edu.cn";
const ATRUST_SUFFIX: &str = ".atrust.nju.edu.cn";

/// 将普通网页请求改写为南京大学 Web VPN 地址。
///
/// 已经位于 aTrust 或 ZTNA 的请求会原样发送，响应不会被修改。
#[derive(Debug, Clone, Copy, Default)]
pub struct WebVpnMiddleware;

#[async_trait]
impl Middleware for WebVpnMiddleware {
    async fn handle(
        &self,
        mut request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        *request.url_mut() = routed_url(request.url()).map_err(|error| {
            reqwest_middleware::Error::middleware(std::io::Error::other(error.to_string()))
        })?;
        next.run(request, extensions).await
    }
}

fn routed_url(url: &Url) -> anyhow::Result<Url> {
    let host = url.host_str();
    if host == Some(ZTNA_HOST) || host.is_some_and(|host| host.ends_with(ATRUST_SUFFIX)) {
        return Ok(url.clone());
    }

    web_vpn_url(url)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use reqwest::Url;

    use super::routed_url;

    #[test]
    fn rewrites_an_original_url() -> Result<()> {
        let url = Url::parse("https://foo-bar.nju.edu.cn/a?q=1")?;
        assert_eq!(
            routed_url(&url)?.as_str(),
            "https://foo--bar-nju-edu-cn.atrust.nju.edu.cn/a?q=1"
        );
        Ok(())
    }

    #[test]
    fn leaves_web_vpn_urls_unchanged() -> Result<()> {
        for url in [
            "https://foo-nju-edu-cn.atrust.nju.edu.cn/a",
            "https://ztna.nju.edu.cn/portal/shortcut.html",
        ] {
            let url = Url::parse(url)?;
            assert_eq!(routed_url(&url)?, url);
        }
        Ok(())
    }
}
