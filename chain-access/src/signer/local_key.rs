use alloy::consensus::{SignableTransaction, TxEip1559};
use alloy::eips::eip2718::Encodable2718;
use alloy::network::TxSignerSync;
use alloy::primitives::Signature;
use alloy::primitives::{Address, Bytes, TxKind, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use async_trait::async_trait;

use crate::error::ChainAccessError;
use crate::signer::SignerBackend;

/// Signer backed by a raw secp256k1 private key loaded from an environment variable.
#[derive(Debug)]
pub struct LocalKeySigner {
    key: PrivateKeySigner,
}

impl LocalKeySigner {
    /// Load from a hex private key string (`0x`-prefix optional).
    pub fn from_hex(hex: &str) -> Result<Self, ChainAccessError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let key = hex
            .parse::<PrivateKeySigner>()
            .map_err(|e| ChainAccessError::Signer(format!("invalid private key: {e}")))?;
        Ok(Self { key })
    }

    /// Load from an environment variable that contains a hex private key.
    pub fn from_env(var: &str) -> Result<Self, ChainAccessError> {
        let val = std::env::var(var)
            .map_err(|_| ChainAccessError::Signer(format!("env var `{var}` not set")))?;
        Self::from_hex(&val)
    }
}

#[async_trait]
impl SignerBackend for LocalKeySigner {
    async fn address(&self) -> Result<Address, ChainAccessError> {
        Ok(self.key.address())
    }

    async fn sign_transaction(&self, tx: TransactionRequest) -> Result<Bytes, ChainAccessError> {
        let err = |field: &str| {
            ChainAccessError::TxBuild(format!(
                "missing required field `{field}` in TransactionRequest"
            ))
        };

        let to = tx.to.unwrap_or(TxKind::Create);

        let mut typed_tx = TxEip1559 {
            chain_id: tx.chain_id.ok_or_else(|| err("chain_id"))?,
            nonce: tx.nonce.ok_or_else(|| err("nonce"))?,
            gas_limit: tx.gas.ok_or_else(|| err("gas"))?,
            max_fee_per_gas: tx.max_fee_per_gas.ok_or_else(|| err("max_fee_per_gas"))?,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas.unwrap_or(0),
            to,
            value: tx.value.unwrap_or(U256::ZERO),
            access_list: Default::default(),
            input: tx.input.into_input().unwrap_or_default(),
        };

        let sig: Signature = self
            .key
            .sign_transaction_sync(&mut typed_tx)
            .map_err(|e| ChainAccessError::Signer(e.to_string()))?;

        let signed = typed_tx.into_signed(sig);
        let mut buf = Vec::new();
        signed.encode_2718(&mut buf);
        Ok(Bytes::from(buf))
    }

    fn signer_kind(&self) -> &'static str {
        "local-key"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    #[tokio::test]
    async fn test_from_hex_with_prefix() {
        let s = LocalKeySigner::from_hex(TEST_KEY).unwrap();
        let addr = s.address().await.unwrap();
        assert_eq!(addr.to_string().to_lowercase(), TEST_ADDR);
    }

    #[tokio::test]
    async fn test_from_hex_without_prefix() {
        let key = TEST_KEY.strip_prefix("0x").unwrap();
        let s = LocalKeySigner::from_hex(key).unwrap();
        let addr = s.address().await.unwrap();
        assert_eq!(addr.to_string().to_lowercase(), TEST_ADDR);
    }

    #[test]
    fn test_from_env_missing() {
        let result = LocalKeySigner::from_env("__STABLECHAIN_NONEXISTENT_ENV_VAR__");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not set"));
    }

    #[test]
    fn test_signer_kind() {
        let s = LocalKeySigner::from_hex(TEST_KEY).unwrap();
        assert_eq!(s.signer_kind(), "local-key");
    }
}
