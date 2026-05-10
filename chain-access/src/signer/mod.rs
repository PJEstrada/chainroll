pub mod local_key;
pub mod privy_signer;
pub mod privy_user_signer;

pub use local_key::LocalKeySigner;
pub use privy_signer::PrivySigner;
pub use privy_user_signer::PrivyUserSigner;

use alloy::primitives::{Address, Bytes};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use std::str::FromStr;

use crate::error::ChainAccessError;

pub enum SignerBackendType {
    LocalKey,
    Privy,
    PrivyUser,
}

impl FromStr for SignerBackendType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local-key" => Ok(Self::LocalKey),
            "privy" => Ok(Self::Privy),
            "privy-user" => Ok(Self::PrivyUser),
            _ => Err(()),
        }
    }
}

/// Abstraction over a signing backend.
/// Implementations sign a `TransactionRequest` and return the RLP-encoded signed bytes.
/// They know nothing about chains, RPC, or broadcasting.
#[async_trait]
pub trait SignerBackend: Send + Sync {
    /// Returns the Ethereum address controlled by this signer.
    async fn address(&self) -> Result<Address, ChainAccessError>;

    /// Signs the transaction and returns the RLP-encoded signed tx bytes.
    async fn sign_transaction(&self, tx: TransactionRequest) -> Result<Bytes, ChainAccessError>;

    /// A short label for display purposes (e.g. `"local-key"`, `"privy"`, `"privy-user"`).
    fn signer_kind(&self) -> &'static str;
}
