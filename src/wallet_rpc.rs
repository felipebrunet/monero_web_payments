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

        if let Some(err) = resp.error {
            return Err(anyhow!("RPC Error {}: {}", err.code, err.message));
        }

        resp.result.ok_or_else(|| anyhow!("Missing result in RPC response"))
    }



    /// Create a new subaddress (account 0)
    pub async fn create_subaddress(&self) -> Result<SubaddressResult> {
        self.call::<_, SubaddressResult>(
            "create_address",
            CreateAddressParams {
                account_index: 0,
                label: "",
            },
        )
        .await
    }

    /// Get incoming transfers for a specific subaddress index
    pub async fn get_transfers(&self, address_index: u32) -> Result<Vec<TransferEntry>> {
        let params = GetTransfersParams {
            in_: true,
            account_index: 0,
            subaddr_indices: vec![address_index],
            pool: true,
        };

        let res: GetTransfersResult = self.call("get_transfers", params).await?;
        
        let mut transfers = res.in_.unwrap_or_default();
        if let Some(pool) = res.pool {
            transfers.extend(pool);
        }
        Ok(transfers)
    }

    /// Verify a signed message
    pub async fn verify_message(&self, address: &str, message: &str, signature: &str) -> Result<bool> {
        let params = VerifyParams {
            data: message,
            address,
            signature,
        };
        let res: VerifyResult = self.call("verify", params).await?;
        Ok(res.good)
    }
}

/* ---------- RPC PARAMS & RESULTS ---------- */

#[derive(Serialize)]
struct CreateAddressParams<'a> {
    account_index: u32,
    label: &'a str,
}

#[derive(Debug, Deserialize, Default)]
pub struct SubaddressResult {
    pub address: String,
    pub address_index: u32,
}

#[derive(Serialize)]
struct GetTransfersParams {
    #[serde(rename = "in")]
    in_: bool,
    account_index: u32,
    subaddr_indices: Vec<u32>,
    pool: bool,
}

#[derive(Deserialize, Default)]
struct GetTransfersResult {
    #[serde(rename = "in")]
    in_: Option<Vec<TransferEntry>>,
    pool: Option<Vec<TransferEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct TransferEntry {
    pub amount: u64,
    #[serde(default)]
    pub confirmations: u64,
    #[allow(dead_code)]
    pub txid: String,
}

#[derive(Serialize)]
struct VerifyParams<'a> {
    data: &'a str,
    address: &'a str,
    signature: &'a str,
}

#[derive(Deserialize, Default)]
struct VerifyResult {
    good: bool,
}
