use super::SignerBackend;
use crate::error::ChainAccessError;
use alloy::hex;
use alloy::primitives::{Address, Bytes};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use privy_rs::generated::types::{
    EthereumSignTransactionRpcInputParamsTransaction,
    EthereumSignTransactionRpcInputParamsTransactionChainId,
    EthereumSignTransactionRpcInputParamsTransactionGasLimit,
    EthereumSignTransactionRpcInputParamsTransactionMaxFeePerGas,
    EthereumSignTransactionRpcInputParamsTransactionMaxPriorityFeePerGas,
    EthereumSignTransactionRpcInputParamsTransactionNonce,
    EthereumSignTransactionRpcInputParamsTransactionValue, WalletRpcResponse,
};
use privy_rs::{AuthorizationContext, PrivyClient};

pub struct PrivySigner {
    client: PrivyClient,
    wallet_id: String,
}

impl PrivySigner {
    pub fn new(client: PrivyClient, wallet_id: String) -> Self {
        Self { client, wallet_id }
    }
}

#[async_trait]
impl SignerBackend for PrivySigner {
    async fn address(&self) -> Result<Address, ChainAccessError> {
        let wallet = self
            .client
            .wallets()
            .get(&self.wallet_id)
            .await
            .map_err(|e| ChainAccessError::Signer(e.to_string()))?;
        let addr_str = &wallet.address;
        addr_str
            .parse()
            .map_err(|e| ChainAccessError::Signer(format!("invalid wallet address: {e}")))
    }

    /// Signs the transaction and returns the RLP-encoded signed tx bytes.
    async fn sign_transaction(&self, tx: TransactionRequest) -> Result<Bytes, ChainAccessError> {
        let ethereum_service = self.client.wallets().ethereum();
        let auth_ctx = AuthorizationContext::new();
        let privy_tx = to_privy_tx(tx);

        let result = ethereum_service
            .sign_transaction(
                self.wallet_id.as_str(),
                privy_tx,
                &auth_ctx,
                None, // no idempotency key
            )
            .await
            .map_err(|e| ChainAccessError::Signer(e.to_string()))?;

        let response = result.into_inner();

        // Extract signed_transaction from WalletRpcResponse
        let signed_hex = match response {
            WalletRpcResponse::EthereumSignTransactionRpcResponse(resp) => {
                resp.data.signed_transaction
            }
            _ => return Err(ChainAccessError::Signer("unexpected response type".into())),
        };

        // Decode hex string to Bytes
        let hex_clean = signed_hex.strip_prefix("0x").unwrap_or(&signed_hex);
        hex::decode(hex_clean).map(Bytes::from).map_err(|e| {
            ChainAccessError::Signer(format!("invalid hex in signed transaction: {e}"))
        })
    }

    fn signer_kind(&self) -> &'static str {
        "privy"
    }
}

fn to_privy_tx(tx: TransactionRequest) -> EthereumSignTransactionRpcInputParamsTransaction {
    use alloy::primitives::TxKind;
    EthereumSignTransactionRpcInputParamsTransaction {
        to: tx.to.and_then(|t| match t {
            TxKind::Call(addr) => Some(addr.to_string()),
            TxKind::Create => None,
        }),
        value: tx.value.map(|v| {
            EthereumSignTransactionRpcInputParamsTransactionValue::String(format!("{:#x}", v))
        }),
        data: tx.input.into_input().map(|b| format!("{b}")),
        nonce: tx.nonce.map(|n| {
            EthereumSignTransactionRpcInputParamsTransactionNonce::String(format!("{:#x}", n))
        }),
        gas_limit: tx.gas.map(|g| {
            EthereumSignTransactionRpcInputParamsTransactionGasLimit::String(format!("{:#x}", g))
        }),
        max_fee_per_gas: tx.max_fee_per_gas.map(|f| {
            EthereumSignTransactionRpcInputParamsTransactionMaxFeePerGas::String(format!(
                "{:#x}",
                f
            ))
        }),
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas.map(|f| {
            EthereumSignTransactionRpcInputParamsTransactionMaxPriorityFeePerGas::String(format!(
                "{:#x}",
                f
            ))
        }),
        chain_id: tx.chain_id.map(|c| {
            EthereumSignTransactionRpcInputParamsTransactionChainId::String(format!("{:#x}", c))
        }),
        ..Default::default()
    }
}
