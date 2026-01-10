use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::{JsonRpcRequest, JsonRpcResponse};

/// Thin client over monero-wallet-rpc JSON-RPC
pub struct WalletRpc {
    client: Client,
    url: String,
}

impl WalletRpc {
    /// Create a new Wallet RPC client
    ///
    /// Auth intentionally ignored for now (added later cleanly)
    pub fn new(
        url: String,
        _user: Option<String>,
        _password: Option<String>,
    ) -> Result<Self> {
        let client = Client::builder().build()?;

        // Ensure /json_rpc is always used
        let url = if url.ends_with("/json_rpc") {
            url
        } else {
            format!("{}/json_rpc", url.trim_end_matches('/'))
        };

        Ok(Self { client, url })
    }

    /// Generic JSON-RPC call
    async fn call<T, R>(&self, method: &'static str, params: T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de> + Default,
    {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: "0",
            method,
            params,
        };

        let resp = self
            .client
            .post(&self.url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<JsonRpcResponse<R>>()
            .await?;

        Ok(resp.result.unwrap_or_default())
    }

    /// Open an existing wallet (view-only is fine)
    pub async fn open_wallet(&self, filename: &str) -> Result<()> {
        self.call::<_, EmptyResult>(
            "open_wallet",
            OpenWalletParams { filename },
        )
        .await
        .map_err(|e| anyhow!("Failed to open wallet: {}", e))?;

        Ok(())
    }
}

/* ---------- RPC PARAMS & RESULTS ---------- */

#[derive(Serialize)]
struct OpenWalletParams<'a> {
    filename: &'a str,
}

#[derive(Deserialize, Default)]
struct EmptyResult;
