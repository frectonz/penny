mod api;
mod auth;
mod collector;
mod config;
mod db;
mod proxy;
mod reporter;
mod types;

use clap::{Parser, Subcommand};
use tracing::info;

use api::create_api_router;
use config::Config;
use db::SqliteDatabase;
use proxy::YarpProxy;

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

        /// Password for dashboard access (can also use PENNY_PASSWORD env var)
        #[arg(long, env = "PENNY_PASSWORD")]
        password: Option<String>,
    },
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,penny=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let Args {
        command:
            Command::Serve {
                config,
                address,
                password,
            },
    } = Args::parse();

    auth::init_password(password.clone());
    info!(
        config = %config,
        address = %address,
        auth_enabled = password.is_some(),
        "starting penny proxy"
    );

    let mut server = pingora::server::Server::new(None).unwrap();
    server.bootstrap();

    let config_content = std::fs::read_to_string(&config)?;
    let config: Config = toml::from_str(&config_content)?;

    info!(apps_count = config.apps.len(), "loaded configuration");
    for (host, app) in &config.apps {
        let app = app.blocking_read();
        info!(
            host = %host,
            address = %app.address,
            health_check = %app.health_check,
            "registered app"
        );
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let collector = runtime.block_on(SqliteDatabase::new(&config.database_url))?;

    if let Some(api_address) = config.api_address {
        let api_collector = collector.clone();
        runtime.spawn(async move {
            let router = create_api_router(api_collector);
            let listener = tokio::net::TcpListener::bind(api_address).await.unwrap();
            info!(address = %api_address, "API server listening");
            axum::serve(listener, router).await.unwrap();
        });
    }

    let proxy = YarpProxy::new(config, collector);

    let mut proxy_service = pingora::prelude::http_proxy_service(&server.configuration, proxy);
    proxy_service.add_tcp(&address);

    server.add_service(proxy_service);

    info!(address = %address, "proxy server listening");
    server.run_forever()
}
