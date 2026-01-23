use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::collector::Collector;
use crate::config::{App, Config};
use crate::types::Host;

pub struct YarpProxy<C> {
    pub config: Config,
    pub collector: C,
}

impl<C> YarpProxy<C>
where
    C: Collector,
{
    pub fn new(config: Config, collector: C) -> Self {
        Self { config, collector }
    }
}

pub fn get_host(session: &pingora::prelude::Session) -> Option<&str> {
    session
        .get_header(http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|host| host.split(':').next())
        .or(session.req_header().uri.host())
}

pub struct ProxyContext {
    pub host: Host,
    pub app: Arc<RwLock<App>>,
    pub peer: Box<pingora::prelude::HttpPeer>,
}

impl ProxyContext {
    pub async fn new(host: &str, app: Arc<RwLock<App>>) -> Self {
        let address = app.read().await.address;

        Self {
            app,
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
        let host = get_host(session).ok_or_else(|| {
            warn!("request missing host header");
            pingora::Error::explain(pingora::ErrorType::InvalidHTTPHeader, "failed to get host")
        })?;

        debug!(host = %host, "processing request");
        *ctx = self.config.get_proxy_context(host).await;

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

        App::start_app(&ctx.host, &ctx.app, self.collector.clone()).await?;
        App::schedule_kill(&ctx.host, &ctx.app, self.collector.clone()).await;

        let address = ctx.app.read().await.address;
        debug!(host = %ctx.host, upstream = %address, "connecting to upstream");

        Ok(ctx.peer.clone())
    }
}
