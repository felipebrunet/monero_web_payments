use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "moneromerchd")]
#[command(about = "Monero e-commerce payment daemon")]
pub struct Config {
    /// URL of the monero-wallet-rpc
    #[arg(long, default_value = "http://127.0.0.1:18083")]
    pub wallet_rpc_url: String,

    /// Optional username for monero-wallet-rpc authentication
    #[arg(long)]
    pub wallet_rpc_user: Option<String>,

    /// Optional password for monero-wallet-rpc authentication
    #[arg(long)]
    pub wallet_rpc_password: Option<String>,

    /// Directory where the wallet files are stored (informational)
    #[arg(long, default_value = "wallet")]
    pub wallet_dir: String,

    /// Address to listen on (e.g., 0.0.0.0:8080)
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub listen: String,
}
