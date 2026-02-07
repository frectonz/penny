use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::challenge::{ChallengeStore, get_challenge};
use crate::collector::Collector;
use crate::config::{App, Config};
use crate::types::Host;

pub struct YarpProxy<C> {
    pub config: Config,
    pub collector: C,
    pub challenge_store: ChallengeStore,
}

impl<C> YarpProxy<C>
where
    C: Collector,
{
    pub fn new(config: Config, collector: C, challenge_store: ChallengeStore) -> Self {
        Self {
            config,
            collector,
            challenge_store,
        }
    }
}

/// Responds to an ACME HTTP-01 challenge request.
async fn respond_to_acme_challenge(
    session: &mut pingora::proxy::Session,
    key_auth: &str,
) -> pingora::Result<bool> {
    let mut resp = pingora::http::ResponseHeader::build(200, None)?;
    resp.insert_header(http::header::CONTENT_TYPE, "text/plain")?;
    resp.insert_header(http::header::CONTENT_LENGTH, key_auth.len().to_string())?;

    session.write_response_header(Box::new(resp), false).await?;
    session
        .write_response_body(Some(Bytes::from(key_auth.to_owned())), true)
        .await?;

    Ok(true)
}

pub fn get_host(session: &pingora::prelude::Session) -> Option<&str> {
    session
        .get_header(http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|host| host.split(':').next())
        .or(session.req_header().uri.host())
}

fn is_browser_navigation(session: &pingora::prelude::Session) -> bool {
    // Must be GET
    if session.req_header().method != http::Method::GET {
        return false;
    }

    // Must accept text/html (browsers always send this for navigation)
    let accepts_html = session
        .get_header(http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("text/html"))
        .unwrap_or(false);
    if !accepts_html {
        return false;
    }

    // Sec-Fetch-Dest (set by modern browsers, unforgeable by JS):
    // "document" = page navigation, "empty" = fetch/XHR, "image"/"script"/"style" = sub-resources
    if let Some(dest) = session
        .get_header("Sec-Fetch-Dest")
        .and_then(|v| v.to_str().ok())
        && dest != "document"
    {
        return false;
    }

    // Sec-Fetch-Mode: "navigate" = page navigation, "cors"/"no-cors" = fetch/XHR
    if let Some(mode) = session
        .get_header("Sec-Fetch-Mode")
        .and_then(|v| v.to_str().ok())
        && mode != "navigate"
    {
        return false;
    }

    // Reject WebSocket upgrade requests
    if session.get_header(http::header::UPGRADE).is_some() {
        return false;
    }

    true
}

fn loading_page_html(host: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="refresh" content="2">
    <title>Starting {host}...</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background: #fafafa;
            color: #0a0a0a;
        }}
        @media (prefers-color-scheme: dark) {{
            body {{ background: #0a0a0a; color: #fafafa; }}
            .github-link {{ color: #a1a1a1; }}
            .github-link:hover {{ color: #f97316; }}
        }}
        .container {{ text-align: center; padding: 2rem; }}
        .logo {{
            animation: pulse 1s ease-in-out infinite;
        }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; transform: scale(1); }}
            50% {{ opacity: 0.6; transform: scale(0.95); }}
        }}
        h1 {{
            font-size: 1.25rem;
            font-weight: 500;
            margin-top: 1.5rem;
            margin-bottom: 0.5rem;
        }}
        .subtitle {{
            font-size: 0.875rem;
            color: #888;
            margin-bottom: 1.5rem;
        }}
        .github-link {{
            font-size: 0.75rem;
            color: #888;
            text-decoration: none;
        }}
        .github-link:hover {{ color: #f97316; }}
    </style>
</head>
<body>
    <div class="container">
        <svg class="logo" width="80" height="80" viewBox="0 0 100 100" fill="none"
             xmlns="http://www.w3.org/2000/svg" role="img">
            <title>Penny Logo</title>
            <circle cx="50" cy="50" r="45" stroke="currentColor" stroke-width="6" fill="none" />
            <circle cx="50" cy="50" r="35" stroke="currentColor" stroke-width="3" fill="none" />
            <text x="50" y="58" text-anchor="middle" fill="currentColor"
                  font-size="36" font-weight="bold" font-family="system-ui, sans-serif">P</text>
        </svg>
        <h1>Starting {host}</h1>
        <p class="subtitle">This page will refresh automatically.</p>
        <a class="github-link" href="https://github.com/frectonz/penny"
           target="_blank" rel="noopener noreferrer">github.com/frectonz/penny</a>
    </div>
</body>
</html>"#,
        host = host
    )
}

async fn respond_with_loading_page(
    session: &mut pingora::proxy::Session,
    host: &str,
) -> pingora::Result<bool> {
    let body = loading_page_html(host);
    let mut resp = pingora::http::ResponseHeader::build(202, None)?;
    resp.insert_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")?;
    resp.insert_header(http::header::CONTENT_LENGTH, body.len().to_string())?;
    resp.insert_header(http::header::CACHE_CONTROL, "no-store")?;
    resp.insert_header("Refresh", "2")?;

    session.write_response_header(Box::new(resp), false).await?;
    session
        .write_response_body(Some(Bytes::from(body)), true)
        .await?;

    Ok(true)
}

pub struct ProxyContext {
    pub host: Host,
    pub app: Option<Arc<RwLock<App>>>,
    pub peer: Box<pingora::prelude::HttpPeer>,
}

impl ProxyContext {
    pub async fn new(host: &str, app: Arc<RwLock<App>>) -> Self {
        let address = app.read().await.address;

        Self {
            app: Some(app),
            host: Host(host.to_owned()),
            peer: Box::new(pingora::prelude::HttpPeer::new(
                address,
                false,
                host.to_owned(),
            )),
        }
    }

    pub fn new_api(host: &str, address: std::net::SocketAddr) -> Self {
        Self {
            app: None,
            host: Host(host.to_owned()),
            peer: Box::new(pingora::prelude::HttpPeer::new(
                address,
                false,
                host.to_owned(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl<C> pingora::prelude::ProxyHttp for YarpProxy<C>
where
    C: Collector,
{
    type CTX = Option<ProxyContext>;

    fn new_ctx(&self) -> Self::CTX {
        None
    }

    async fn request_filter(
        &self,
        session: &mut pingora::prelude::Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        let path = session.req_header().uri.path();

        // Check for ACME challenge requests before normal routing
        if path.starts_with("/.well-known/acme-challenge/") {
            let token = path.trim_start_matches("/.well-known/acme-challenge/");
            if let Some(key_auth) = get_challenge(&self.challenge_store, token).await {
                debug!(token = %token, "responding to ACME challenge");
                return respond_to_acme_challenge(session, &key_auth).await;
            }
        }

        let host = get_host(session).ok_or_else(|| {
            warn!("request missing host header");
            pingora::Error::explain(pingora::ErrorType::InvalidHTTPHeader, "failed to get host")
        })?;

        debug!(host = %host, "processing request");
        *ctx = self.config.get_proxy_context(host).await;

        if let Some(proxy_ctx) = ctx.as_ref()
            && let Some(app) = &proxy_ctx.app
        {
            let cold_start_page = app.read().await.cold_start_page;
            if cold_start_page && is_browser_navigation(session) {
                let is_ready =
                    App::begin_start_app(&proxy_ctx.host, app, self.collector.clone()).await?;
                App::schedule_kill(&proxy_ctx.host, app, self.collector.clone()).await;
                if !is_ready {
                    return respond_with_loading_page(session, &proxy_ctx.host.0).await;
                }
            }
        }

        if ctx.is_none() {
            warn!(host = %host, "no app configured for host");
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut pingora::proxy::Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<pingora::prelude::HttpPeer>> {
        let ctx = ctx.take().ok_or_else(|| {
            error!("no proxy context available");
            pingora::Error::explain(
                pingora::ErrorType::ConnectError,
                "failed to get proxy context",
            )
        })?;

        info!(host = %ctx.host, "proxying request");

        if let Some(ref app) = ctx.app {
            App::start_app(&ctx.host, app, self.collector.clone()).await?;
            App::schedule_kill(&ctx.host, app, self.collector.clone()).await;
        }

        Ok(ctx.peer.clone())
    }
}
