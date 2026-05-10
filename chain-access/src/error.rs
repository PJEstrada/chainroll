use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainAccessError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("signer error: {0}")]
    Signer(String),

    #[error("transaction build error: {0}")]
    TxBuild(String),

    #[error("unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
