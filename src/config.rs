use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "moneromerchd")]
#[command(about = "Monero e-commerce payment daemon")]
pub struct Config {
    #[arg(long, default_value = "http://127.0.0.1:18083")]
    pub wallet_rpc_url: String,

    #[arg(long)]
    pub wallet_rpc_user: Option<String>,

    #[arg(long)]
    pub wallet_rpc_password: Option<String>,

    #[arg(long, default_value = "wallet")]
    pub wallet_dir: String,

    #[arg(long, default_value = "127.0.0.1:8080")]
    pub listen: String,
}
