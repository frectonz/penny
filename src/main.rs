mod acme;
mod api;
mod auth;
mod challenge;
mod collector;
mod config;
mod db;
mod proxy;
mod reporter;
mod tls;
mod types;

use clap::{Parser, Subcommand};
use tracing::{error, info, warn};

use acme::AcmeClient;
use api::create_api_router;
use challenge::{ChallengeStore, create_challenge_store};
use config::{Config, TlsConfig};
use db::SqliteDatabase;
use proxy::YarpProxy;
use tls::CertificateStore;

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

        /// The HTTP address to bind to.
        #[arg(short, long, default_value = "0.0.0.0:80")]
        address: String,

        /// The HTTPS address to bind to.
        #[arg(long, default_value = "0.0.0.0:443")]
        https_address: String,

        /// Disable TLS even if configured.
        #[arg(long)]
        no_tls: bool,

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
                https_address,
                no_tls,
                password,
            },
    } = Args::parse();

    auth::init_password(password.clone());
    info!(
        config = %config,
        address = %address,
        https_address = %https_address,
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

    // Create challenge store for ACME HTTP-01 challenges
    let challenge_store = create_challenge_store();

    // Check if TLS is enabled and extract configuration before moving config
    let tls_enabled = config.tls.as_ref().is_some_and(|t| t.enabled) && !no_tls;
    let tls_config = config.tls.clone();
    let domains: Vec<String> = config.apps.keys().cloned().collect();

    if tls_enabled {
        let tls_config = tls_config.as_ref().unwrap();

        if domains.is_empty() {
            warn!("TLS enabled but no apps configured, skipping certificate provisioning");
        } else {
            // Provision certificates
            runtime.block_on(provision_certificates(
                &domains,
                &collector,
                &challenge_store,
                tls_config,
            ))?;

            // Spawn renewal background task
            let renewal_collector = collector.clone();
            let renewal_store = challenge_store.clone();
            let renewal_config = tls_config.clone();
            let renewal_domains = domains.clone();

            runtime.spawn(async move {
                renewal_loop(
                    renewal_domains,
                    renewal_collector,
                    renewal_store,
                    renewal_config,
                )
                .await;
            });
        }
    }

    let proxy = YarpProxy::new(config, collector, challenge_store);

    let mut proxy_service = pingora::prelude::http_proxy_service(&server.configuration, proxy);

    // Always add HTTP listener (needed for ACME challenges and non-TLS traffic)
    proxy_service.add_tcp(&address);
    info!(address = %address, "HTTP proxy server listening");

    // Add HTTPS listener if TLS is enabled
    if tls_enabled && !domains.is_empty() {
        let tls_config = tls_config.as_ref().unwrap();
        let cert_store = CertificateStore::new(&tls_config.certs_dir)?;

        // Use the first domain's certificate as default (SNI will be handled by OpenSSL)
        if let Some((cert_path, key_path)) = cert_store.get_certificate(&domains[0]) {
            let tls_settings = pingora::listeners::tls::TlsSettings::intermediate(
                cert_path.to_str().unwrap(),
                key_path.to_str().unwrap(),
            )?;

            proxy_service.add_tls_with_settings(&https_address, None, tls_settings);
            info!(address = %https_address, "HTTPS proxy server listening");
        } else {
            warn!("no certificates available, HTTPS listener not started");
        }
    }

    server.add_service(proxy_service);

    server.run_forever()
}

/// Provisions certificates for all domains that need them.
async fn provision_certificates(
    domains: &[String],
    db: &SqliteDatabase,
    challenge_store: &ChallengeStore,
    tls_config: &TlsConfig,
) -> color_eyre::Result<()> {
    let cert_store = CertificateStore::new(&tls_config.certs_dir)?;
    let acme_client = AcmeClient::new(tls_config, db).await?;

    for domain in domains {
        if cert_store.needs_renewal(domain, tls_config.renewal_days) {
            info!(domain = %domain, "provisioning certificate");

            match acme_client
                .obtain_certificate(&[domain.as_str()], challenge_store)
                .await
            {
                Ok((cert_pem, key_pem)) => {
                    cert_store.store_certificate(domain, &cert_pem, &key_pem)?;
                    info!(domain = %domain, "certificate provisioned successfully");
                }
                Err(e) => {
                    error!(domain = %domain, error = %e, "failed to provision certificate");
                }
            }
        } else {
            info!(domain = %domain, "certificate valid, skipping provisioning");
        }
    }

    Ok(())
}

/// Background task that periodically checks for certificates needing renewal.
async fn renewal_loop(
    domains: Vec<String>,
    db: SqliteDatabase,
    challenge_store: ChallengeStore,
    tls_config: TlsConfig,
) {
    // Check for renewal every 12 hours
    let check_interval = std::time::Duration::from_secs(12 * 60 * 60);

    loop {
        tokio::time::sleep(check_interval).await;

        info!("checking certificates for renewal");

        if let Err(e) = provision_certificates(&domains, &db, &challenge_store, &tls_config).await {
            error!(error = %e, "certificate renewal check failed");
        }
    }
}
