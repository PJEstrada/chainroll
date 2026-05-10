use alloy::primitives::{Bytes, TxHash};
use alloy::rpc::types::TransactionReceipt;
use async_trait::async_trait;

use crate::error::ChainAccessError;

/// Write access to a single chain — broadcast and confirm transactions.
#[async_trait]
pub trait ChainWriter: Send + Sync {
    async fn send_raw_transaction(&self, rlp: Bytes) -> Result<TxHash, ChainAccessError>;

    async fn wait_for_receipt(
        &self,
        tx_hash: &TxHash,
    ) -> Result<TransactionReceipt, ChainAccessError>;
}
