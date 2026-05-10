use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;

use crate::domain::chain_id::ChainId;
use crate::error::ChainAccessError;

/// Read-only access to a single chain.
#[async_trait]
pub trait ChainReader: Send + Sync {
    fn chain_id(&self) -> ChainId;

    async fn native_balance(&self, address: Address) -> Result<U256, ChainAccessError>;

    async fn erc20_balance(&self, token: Address, owner: Address)
    -> Result<U256, ChainAccessError>;

    async fn nonce(&self, address: Address) -> Result<u64, ChainAccessError>;

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<u64, ChainAccessError>;

    async fn gas_price(&self) -> Result<u128, ChainAccessError>;

    async fn block_number(&self) -> Result<u64, ChainAccessError>;
}
