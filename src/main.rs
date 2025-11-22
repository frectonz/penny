use clap::{Parser, Subcommand};

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


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    dbg!(args);

    Ok(())
}
