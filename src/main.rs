use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;

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

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub address: std::net::SocketAddr,
    pub domain: String,
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


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let Args { command: Command::Serve { config, address } } = Args::parse();

    let config = toml::from_str::<Config>(&std::fs::read_to_string(&config)?)?;
    dbg!(config);

    Ok(())
}
