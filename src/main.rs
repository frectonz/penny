mod acme;
mod api;
mod auth;
mod challenge;
mod check;
mod collector;
mod config;
mod db;
mod proxy;
mod reporter;
mod systemd;
mod tls;
mod types;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Context;
use tracing::{error, info, warn};

use acme::AcmeClient;
use api::{PaginationConfig, create_api_router};
use challenge::{ChallengeStore, create_challenge_store};
use config::{Config, TlsConfig};
use db::SqliteDatabase;
use proxy::YarpProxy;
use tls::{CertificateStore, DynamicCertificates};

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
    /// Check app start/stop commands by running them.
    Check {
        /// Path to the config file.
        config: String,

        /// Optional list of specific apps to check (by hostname).
        #[arg(long, value_delimiter = ',')]
        apps: Option<Vec<String>>,
    },
    /// Manage penny as a systemd user service.
    Systemd {
        #[clap(subcommand)]
        action: SystemdAction,
    },
}

#[derive(Debug, Subcommand)]
enum SystemdAction {
    /// Install and start the penny systemd user service.
    Install {
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
    /// Stop and remove the penny systemd user service.
    Uninstall,
    /// Show the status of the penny systemd user service.
    Status,
    /// Show logs from the penny systemd user service.
    Logs {
        /// Follow log output.
        #[arg(short, long)]
        follow: bool,
    },
    /// Restart the penny systemd user service.
    Restart,
}

async fn setup_api_server(
    api_address: Option<std::net::SocketAddr>,
    collector: SqliteDatabase,
    pagination_config: PaginationConfig,
) {
    if let Some(api_address) = api_address {
        let router = create_api_router(collector, pagination_config);
        let listener = tokio::net::TcpListener::bind(api_address).await.unwrap();
        info!(address = %api_address, "API server listening");
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
    }
}

fn setup_tls(
    domains: Vec<String>,
    collector: SqliteDatabase,
    challenge_store: ChallengeStore,
    tls_config: TlsConfig,
) {
    if domains.is_empty() {
        warn!("TLS enabled but no apps configured, skipping certificate provisioning");
        return;
    }

    tokio::spawn(async move {
        if let Err(e) =
            provision_certificates(&domains, &collector, &challenge_store, &tls_config).await
        {
            error!(error = %e, "initial certificate provisioning failed");
        }

        renewal_loop(domains, collector, challenge_store, tls_config).await;
    });
}

async fn setup(
    config: &Config,
    no_tls: bool,
) -> color_eyre::Result<(SqliteDatabase, ChallengeStore)> {
    let collector = SqliteDatabase::new(&config.database_url).await?;
    let pagination_config = PaginationConfig {
        default_limit: config.default_page_limit,
        max_limit: config.max_page_limit,
    };
    setup_api_server(config.api_address, collector.clone(), pagination_config).await;
    let challenge_store = create_challenge_store();

    if let Some(tls_config) = &config.tls
        && tls_config.enabled
        && !no_tls
    {
        let domains = config.tls_domains();
        setup_tls(
            domains,
            collector.clone(),
            challenge_store.clone(),
            tls_config.clone(),
        );
    }

    Ok((collector, challenge_store))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,penny=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    match args.command {
        Command::Check { config, apps } => {
            let runtime = tokio::runtime::Runtime::new().context("creating tokio runtime")?;
            runtime.block_on(check::run_check(&config, apps))?;
            Ok(())
        }
        Command::Systemd { action } => match action {
            SystemdAction::Install {
                config,
                address,
                https_address,
                no_tls,
                password,
            } => systemd::install(systemd::InstallOpts {
                config,
                address,
                https_address,
                no_tls,
                password,
            }),
            SystemdAction::Uninstall => systemd::uninstall(),
            SystemdAction::Status => systemd::status(),
            SystemdAction::Logs { follow } => systemd::logs(follow),
            SystemdAction::Restart => systemd::restart(),
        },
        Command::Serve {
            config,
            address,
            https_address,
            no_tls,
            password,
        } => {
            auth::init_password(password.clone());
            info!(
                config = %config,
                address = %address,
                https_address = %https_address,
                auth_enabled = password.is_some(),
                "starting penny proxy"
            );

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

            if let Some(api_domain) = &config.api_domain {
                if config.api_address.is_some() {
                    info!(api_domain = %api_domain, "API domain configured for proxy routing");
                } else {
                    warn!(api_domain = %api_domain, "api_domain is set but api_address is not configured");
                }
            }

            let mut server =
                pingora::server::Server::new(None).context("creating pingora server")?;
            server.bootstrap();

            let runtime = tokio::runtime::Runtime::new().context("creating tokio runtime")?;
            let (collector, challenge_store) = runtime.block_on(setup(&config, no_tls))?;

            let tls_enabled = config.tls.as_ref().is_some_and(|t| t.enabled) && !no_tls;
            let tls_config = config.tls.clone();
            let domains = config.tls_domains();

            let proxy = YarpProxy::new(config, collector, challenge_store);
            let mut proxy_service =
                pingora::prelude::http_proxy_service(&server.configuration, proxy);

            proxy_service.add_tcp(&address);
            info!(address = %address, "HTTP proxy server listening");

            if tls_enabled && !domains.is_empty() {
                let tls_config = tls_config.as_ref().unwrap();
                let cert_store = CertificateStore::new(&tls_config.certs_dir)?;
                let dynamic_certs = DynamicCertificates::new(cert_store);
                let tls_settings =
                    pingora::listeners::tls::TlsSettings::with_callbacks(Box::new(dynamic_certs))?;

                proxy_service.add_tls_with_settings(&https_address, None, tls_settings);
                info!(address = %https_address, "HTTPS proxy server listening");
            }

            server.add_service(proxy_service);
            server.run_forever()
        }
    }
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
    let check_interval =
        std::time::Duration::from_secs(tls_config.renewal_check_interval_hours * 60 * 60);

    loop {
        tokio::time::sleep(check_interval).await;

        info!("checking certificates for renewal");

        if let Err(e) = provision_certificates(&domains, &db, &challenge_store, &tls_config).await {
            error!(error = %e, "certificate renewal check failed");
        }
    }
}
