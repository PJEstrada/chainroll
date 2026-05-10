pub mod tempo_adapter;
pub mod tempo_provider;

pub use tempo_adapter::TempoAdapter;

use crate::domain::chain_id::ChainId;
use crate::error::ChainAccessError;
use crate::ports::{ChainReader, ChainWriter};

/// Returns the correct [`ChainReader`] implementation for the given chain.
/// Add a new arm here when a new chain is supported.
pub async fn connect_reader(chain_id: ChainId) -> Result<Box<dyn ChainReader>, ChainAccessError> {
    match chain_id {
        ChainId::TempoTestnet => Ok(Box::new(TempoAdapter::connect(chain_id).await?)),
        // ChainId::ArcMainnet => Ok(Box::new(ArcAdapter::connect(chain_id).await?)),
    }
}

/// Returns the correct [`ChainWriter`] implementation for the given chain.
/// Add a new arm here when a new chain is supported.
pub async fn connect_writer(chain_id: ChainId) -> Result<Box<dyn ChainWriter>, ChainAccessError> {
    match chain_id {
        ChainId::TempoTestnet => Ok(Box::new(TempoAdapter::connect(chain_id).await?)),
        // ChainId::ArcMainnet => Ok(Box::new(ArcAdapter::connect(chain_id).await?)),
    }
}
