use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};

use crate::domain::erc20;
use crate::error::ChainAccessError;
use crate::ports::{ChainReader, ChainWriter};
use crate::signer::SignerBackend;

/// Orchestrates the full send-flow: nonce → build → estimate gas → sign → broadcast → wait.
///
/// Generic over all three port traits — no concrete adapter or signer types leak in.
/// Designed to be extractable to a separate crate without modification.
pub struct TxExecutor<R, W, S> {
    reader: R,
    writer: W,
    signer: S,
}

impl<R: ChainReader, W: ChainWriter, S: SignerBackend> TxExecutor<R, W, S> {
    pub fn new(reader: R, writer: W, signer: S) -> Self {
        Self {
            reader,
            writer,
            signer,
        }
    }

    /// Sends a native token transfer.
    pub async fn send_native(
        &self,
        to: Address,
        amount: U256,
    ) -> Result<TransactionReceipt, ChainAccessError> {
        let tx = self.build_tx(to, amount, None).await?;
        self.sign_and_send(tx).await
    }

    /// Sends an ERC-20 `transfer(to, amount)`.
    pub async fn send_erc20(
        &self,
        token: Address,
        to: Address,
        amount: U256,
    ) -> Result<TransactionReceipt, ChainAccessError> {
        let calldata = erc20::transfer_calldata(to, amount);
        let tx = self.build_tx(token, U256::ZERO, Some(calldata)).await?;
        self.sign_and_send(tx).await
    }

    async fn build_tx(
        &self,
        to: Address,
        value: U256,
        input: Option<Bytes>,
    ) -> Result<TransactionRequest, ChainAccessError> {
        let sender = self.signer.address().await?;
        let chain_id = self.reader.chain_id();
        let nonce = self.reader.nonce(sender).await?;
        let gas_price = self.reader.gas_price().await?;

        let mut tx = TransactionRequest::default()
            .with_chain_id(chain_id.info().chain_id())
            .with_from(sender)
            .with_to(to)
            .with_value(value)
            .with_nonce(nonce)
            .with_max_fee_per_gas(gas_price)
            .with_max_priority_fee_per_gas(gas_price / 10);

        if let Some(data) = input {
            tx = tx.with_input(data);
        }

        let gas_limit = self.reader.estimate_gas(&tx).await?;
        Ok(tx.with_gas_limit(gas_limit))
    }

    async fn sign_and_send(
        &self,
        tx: TransactionRequest,
    ) -> Result<TransactionReceipt, ChainAccessError> {
        let rlp = self.signer.sign_transaction(tx).await?;
        let hash = self.writer.send_raw_transaction(rlp).await?;
        self.writer.wait_for_receipt(&hash).await
    }
}
