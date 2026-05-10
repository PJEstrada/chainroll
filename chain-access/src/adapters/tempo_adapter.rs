use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;

use alloy::primitives::TxHash;
use alloy::rpc::types::TransactionReceipt;

use crate::adapters::tempo_provider::{TempoProvider, connect_tempo_url};
use crate::domain::chain_id::ChainId;
use crate::domain::erc20;
use crate::error::ChainAccessError;
use crate::ports::{ChainReader, ChainWriter};

pub struct TempoAdapter {
    provider: TempoProvider,
    chain_id: ChainId,
}

impl TempoAdapter {
    pub async fn connect(chain_id: ChainId) -> Result<Self, ChainAccessError> {
        let url = chain_id.info().rpc_url();
        let provider = connect_tempo_url(url).await?;
        Ok(Self { provider, chain_id })
    }
}

#[async_trait]
impl ChainReader for TempoAdapter {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn native_balance(&self, address: Address) -> Result<U256, ChainAccessError> {
        self.provider
            .get_balance(address)
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn erc20_balance(
        &self,
        token: Address,
        owner: Address,
    ) -> Result<U256, ChainAccessError> {
        let calldata = erc20::balance_of_calldata(owner);
        let tx = TransactionRequest::default()
            .with_to(token)
            .with_input(calldata);
        let raw: Bytes = self
            .provider
            .raw_request("eth_call".into(), (tx, "latest"))
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))?;
        Ok(erc20::decode_u256_return(&raw))
    }

    async fn nonce(&self, address: Address) -> Result<u64, ChainAccessError> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<u64, ChainAccessError> {
        let gas: alloy::primitives::U64 = self
            .provider
            .raw_request("eth_estimateGas".into(), (tx, "latest"))
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))?;
        Ok(gas.to())
    }

    async fn gas_price(&self) -> Result<u128, ChainAccessError> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn block_number(&self) -> Result<u64, ChainAccessError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }
}

#[async_trait]
impl ChainWriter for TempoAdapter {
    async fn send_raw_transaction(&self, rlp: Bytes) -> Result<TxHash, ChainAccessError> {
        self.provider
            .raw_request("eth_sendRawTransaction".into(), (rlp,))
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn wait_for_receipt(
        &self,
        tx_hash: &TxHash,
    ) -> Result<TransactionReceipt, ChainAccessError> {
        const MAX_ATTEMPTS: u32 = 40;
        const POLL_MS: u64 = 500;

        for _ in 0..MAX_ATTEMPTS {
            let receipt: Option<TransactionReceipt> = self
                .provider
                .raw_request("eth_getTransactionReceipt".into(), (tx_hash,))
                .await
                .map_err(|e| ChainAccessError::Rpc(e.to_string()))?;

            if let Some(r) = receipt {
                return Ok(r);
            }

            tokio::time::sleep(std::time::Duration::from_millis(POLL_MS)).await;
        }

        Err(ChainAccessError::Rpc(format!(
            "transaction {tx_hash} not confirmed after {}s",
            MAX_ATTEMPTS * POLL_MS as u32 / 1000
        )))
    }
}
