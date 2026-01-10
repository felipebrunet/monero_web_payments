use serde::{Deserialize, Serialize};

/// Generic JSON-RPC request
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: &'static str,
    pub id: &'static str,
    pub method: &'static str,
    pub params: T,
}

/// Generic JSON-RPC response
///
/// NOTE: Monero often returns `"result": null`
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: String,
    pub result: Option<T>,
}
