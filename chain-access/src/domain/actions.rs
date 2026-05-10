use crate::domain::EVM_ADDRESS_LEN;
use crate::domain::account::AccountRef;
use crate::domain::chain::{Chain, RpcConfig};
use alloy::primitives::U256;

/// These are EVM-specific actions that should work across chains via adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Native token balance via `eth_getBalance`.
    NativeBalance { owner: [u8; EVM_ADDRESS_LEN] },

    /// ERC-20 balance via `balanceOf(owner)`
    Erc20Balance {
        token: [u8; EVM_ADDRESS_LEN],
        owner: [u8; EVM_ADDRESS_LEN],
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecCtx {
    pub chain: Chain,
    pub rpc: RpcConfig,
    pub account: Option<AccountRef>,
}

impl ExecCtx {
    pub fn new(chain: Chain, rpc: RpcConfig, account: Option<AccountRef>) -> Self {
        Self {
            chain,
            rpc,
            account,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecResult {
    /// Returned value for read-only calls.
    Balance(U256),

    /// Opaque transaction identifier returned by the submitter/backend.
    Submitted { tx_id: String },
}
