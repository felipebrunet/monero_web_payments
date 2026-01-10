use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::wallet_rpc::{SubaddressResult, WalletRpc};

#[derive(Clone)]
struct AppState {
    wallet: Arc<WalletRpc>,
}

/* ---------- HTTP TYPES ---------- */

#[derive(Deserialize)]
struct InvoiceRequest {
    amount_xmr: String,
}

#[derive(Serialize)]
struct InvoiceResponse {
    address: String,
    account_index: u32,
    address_index: u32,
    amount_xmr: String,
}

/* ---------- HANDLERS ---------- */

async fn create_invoice(
    State(state): State<AppState>,
    Json(req): Json<InvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let subaddr: SubaddressResult = state
        .wallet
        .create_subaddress()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(InvoiceResponse {
        address: subaddr.address,
        account_index: 0,
        address_index: subaddr.address_index,
        amount_xmr: req.amount_xmr,
    }))
}

/* ---------- SERVER ---------- */

pub async fn run(wallet: WalletRpc) -> anyhow::Result<()> {
    println!("Connecting to monero-wallet-rpc…");
    wallet.open_wallet("merch").await?;
    println!("Wallet opened successfully");

    let state = AppState {
        wallet: Arc::new(wallet),
    };

    let app = Router::new()
        .route("/invoice", post(create_invoice))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
