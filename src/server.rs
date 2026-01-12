use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use tower_http::cors::CorsLayer;

use crate::wallet_rpc::{SubaddressResult, TransferEntry, WalletRpc};

#[derive(Clone)]
struct PriceService {
    client: reqwest::Client,
}

impl PriceService {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("moneromerchd/1.0")
                .build()
                .unwrap(),
        }
    }

    async fn get_xmr_price(&self, currency: &str) -> anyhow::Result<Decimal> {
        let currency = currency.to_lowercase();
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids=monero&vs_currencies={}",
            currency
        );

        #[derive(Deserialize)]
        struct PriceResponse {
            monero: std::collections::HashMap<String, f64>,
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<PriceResponse>()
            .await?;

        let price = resp
            .monero
            .get(&currency)
            .ok_or_else(|| anyhow::anyhow!("Currency not found"))?;

        Decimal::from_f64(*price).ok_or_else(|| anyhow::anyhow!("Invalid price"))
    }
}

#[derive(Clone)]
struct AppState {
    wallet: Arc<WalletRpc>,
    price_service: Arc<PriceService>,
}

/* ---------- HTTP TYPES ---------- */

#[derive(Deserialize)]
struct InvoiceRequest {
    #[serde(alias = "amount_xmr")]
    amount: String,
    #[serde(default = "default_currency")]
    currency: String,
}

fn default_currency() -> String {
    "XMR".to_string()
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
    expected_amount_xmr: String,
}

#[derive(Serialize)]
struct CheckPaymentResponse {
    total_received_xmr: String,
    confirmations: u64,
    tx_count: usize,
    paid: bool,
}

#[derive(Deserialize)]
struct VerifyRequest {
    address: String,
    message: String,
    signature: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    good: bool,
}

/* ---------- HANDLERS ---------- */

async fn create_invoice(
    State(state): State<AppState>,
    Json(req): Json<InvoiceRequest>,
) -> Result<Json<InvoiceResponse>, (StatusCode, String)> {
    let amount = Decimal::from_str(&req.amount).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid amount".to_string()))?;
    if amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, "Amount must be positive".to_string()));
    }

    let currency = req.currency.to_uppercase();
    let amount_xmr = if currency == "XMR" {
        amount
    } else {
        let price = state
            .price_service
            .get_xmr_price(&req.currency)
            .await
            .map_err(|e| {
                eprintln!("Price fetch error: {}", e);
                (StatusCode::BAD_GATEWAY, format!("Price fetch error: {}", e))
            })?;

        if price <= Decimal::ZERO {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid price received".to_string()));
        }
        (amount / price).round_dp(12)
    };

    let subaddr: SubaddressResult = state
        .wallet
        .create_subaddress()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Wallet error: {}", e)))?;

    Ok(Json(InvoiceResponse {
        address: subaddr.address,
        account_index: 0,
        address_index: subaddr.address_index,
        amount_xmr: amount_xmr.to_string(),
    }))
}

async fn check_payment(
    State(state): State<AppState>,
    Json(req): Json<CheckPaymentRequest>,
) -> Result<Json<CheckPaymentResponse>, (StatusCode, String)> {
    let expected = Decimal::from_str(&req.expected_amount_xmr)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid expected amount".to_string()))?;

    let transfers: Vec<TransferEntry> = state
        .wallet
        .get_transfers(req.address_index)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Wallet error: {}", e)))?;

    let total_atomic: u64 = transfers.iter().map(|t| t.amount).sum();
    // Convert atomic units (piconero) to XMR. 1 XMR = 10^12 atomic units
    let total_xmr = Decimal::from(total_atomic) / Decimal::from(1_000_000_000_000_u64);

    Ok(Json(CheckPaymentResponse {
        total_received_xmr: total_xmr.to_string(),
        confirmations: transfers.iter().map(|t| t.confirmations).min().unwrap_or(0),
        tx_count: transfers.len(),
        paid: total_xmr >= expected,
    }))
}

async fn verify_message(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    let good = state
        .wallet
        .verify_message(&req.address, &req.message, &req.signature)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Wallet error: {}", e)))?;

    Ok(Json(VerifyResponse { good }))
}

async fn health_check() -> &'static str {
    "OK"
}

/* ---------- SERVER ---------- */

pub async fn run(wallet: WalletRpc, listen_addr: String) -> anyhow::Result<()> {
    let price_service = PriceService::new();

    let state = AppState {
        wallet: Arc::new(wallet),
        price_service: Arc::new(price_service),
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/invoice", post(create_invoice))
        .route("/check_payment", post(check_payment))
        .route("/verify", post(verify_message))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
