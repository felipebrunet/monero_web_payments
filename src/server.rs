use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use rust_decimal::Decimal;

use crate::wallet_rpc::{SubaddressResult, TransferEntry, WalletRpc};

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

#[derive(Deserialize)]
struct CheckPaymentRequest {
    address_index: u32,
}

#[derive(Serialize)]
struct CheckPaymentResponse {
    total_received_xmr: String,
    confirmations: u64,
    tx_count: usize,
}

/* ---------- HANDLERS ---------- */

async fn create_invoice(
    State(state): State<AppState>,
    Json(req): Json<InvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let amount = Decimal::from_str(&req.amount_xmr).map_err(|_| StatusCode::BAD_REQUEST)?;
    if amount <= Decimal::ZERO {
        return Err(StatusCode::BAD_REQUEST);
    }

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

async fn check_payment(
    State(state): State<AppState>,
    Json(req): Json<CheckPaymentRequest>,
) -> Result<Json<CheckPaymentResponse>, StatusCode> {
    let transfers: Vec<TransferEntry> = state
        .wallet
        .get_transfers(req.address_index)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_atomic: u64 = transfers.iter().map(|t| t.amount).sum();
    // Convert atomic units (piconero) to XMR. 1 XMR = 10^12 atomic units
    let total_xmr = Decimal::from(total_atomic) / Decimal::from(1_000_000_000_000_u64);

    Ok(Json(CheckPaymentResponse {
        total_received_xmr: total_xmr.to_string(),
        confirmations: transfers.iter().map(|t| t.confirmations).max().unwrap_or(0),
        tx_count: transfers.len(),
    }))
}

/* ---------- SERVER ---------- */

pub async fn run(wallet: WalletRpc, listen_addr: String) -> anyhow::Result<()> {

    let state = AppState {
        wallet: Arc::new(wallet),
    };

    let app = Router::new()
        .route("/invoice", post(create_invoice))
        .route("/check_payment", post(check_payment))
        .with_state(state);

    let listener = TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
