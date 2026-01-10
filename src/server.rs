use anyhow::Result;

use crate::config::Config;
use crate::wallet_rpc::WalletRpc;

pub async fn run(config: Config) -> Result<()> {
    let wallet = WalletRpc::new(
        config.wallet_rpc_url,
        config.wallet_rpc_user,
        config.wallet_rpc_password,
    )?;

    println!("Connecting to monero-wallet-rpc…");

    match wallet.open_wallet("merch").await {
        Ok(_) => {
            println!("Wallet opened successfully");
        }
        Err(e) => {
            println!("Failed to open wallet: {e}");
            println!("Make sure the view-only wallet exists and monero-wallet-rpc is running.");
            return Ok(());
        }
    }

    // Stop here for now
    Ok(())
}
