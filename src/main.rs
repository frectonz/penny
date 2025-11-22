use clap::{Parser, Subcommand};
use pingora::ErrorType::InvalidHTTPHeader;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the reverse proxy.
    Serve {
        /// Path to the config file.
        config: String,

        /// The address to bind to.
        #[arg(short, long, default_value = "127.0.0.1:3030")]
        address: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub address: std::net::SocketAddr,
    pub start: String,
    #[serde(default)]
    pub wait_period: Option<u64>,
    #[serde(default)]
    pub health_check: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub apps: HashMap<String, AppConfig>,
}

pub struct YarpProxy {
    config: Arc<Config>,
}

impl YarpProxy {
    fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

#[async_trait::async_trait]
impl pingora::prelude::ProxyHttp for YarpProxy {
    type CTX = Option<AppConfig>;
    fn new_ctx(&self) -> Option<AppConfig> {
        None
    }

    async fn early_request_filter(
        &self,
        session: &mut pingora::prelude::Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        let host = session
            .downstream_session
            .get_header("host")
            .ok_or_else(|| pingora::Error::explain(InvalidHTTPHeader, "No host header detected"))?
            .to_str()
            .map_err(|_| {
                pingora::Error::explain(
                    InvalidHTTPHeader,
                    "Failed to convert host header to string",
                )
            })?;

        *ctx = self.config.apps.get(host).map(|c| c.to_owned());

        Ok(())
    }

    async fn upstream_peer(
        &self,
        _session: &mut pingora::proxy::Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<pingora::prelude::HttpPeer>> {
        todo!("implement upstream_peer")
    }
}

fn main() -> color_eyre::Result<()> {
    let Args {
        command: Command::Serve { config, address },
    } = Args::parse();

    let mut server = pingora::server::Server::new(None).unwrap();
    server.bootstrap();

    let config = std::fs::read_to_string(config)?;
    let config: Config = toml::from_str(&config)?;

    let proxy = YarpProxy::new(config);

    let mut proxy_service = pingora::prelude::http_proxy_service(&server.configuration, proxy);
    proxy_service.add_tcp(&address);

    server.add_service(proxy_service);

    println!("Starting proxy server on {}", address);
    server.run_forever();
}
