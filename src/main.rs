mod config;
mod wallet_rpc;
mod server;
mod types;

use anyhow::Result;
use clap::Parser;

use config::Config;
use wallet_rpc::WalletRpc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    println!("Starting moneromerchd");
    println!("Wallet RPC: {}", config.wallet_rpc_url);
    println!("Listening on {}", config.listen);

    // Create Wallet RPC client
    let wallet = WalletRpc::new(
        config.wallet_rpc_url.clone(),
        config.wallet_rpc_user.clone(),
        config.wallet_rpc_password.clone(),
    )?;

    // Start HTTP server
    server::run(wallet, config.listen).await
}
