mod config;
mod wallet_rpc;
mod fx;
mod invoice;
mod server;
mod types;

use clap::Parser;
use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    println!("Starting moneromerchd");
    println!("Wallet RPC: {}", config.wallet_rpc_url);
    println!("Listening on {}", config.listen);

    server::run(config).await
}
