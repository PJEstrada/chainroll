use alloy::providers::fillers::{FillProvider, JoinFill, NonceFiller};
use alloy::providers::{Identity, ProviderBuilder, RootProvider};
use tempo_alloy::{TempoFillers, TempoNetwork};

use crate::error::ChainAccessError;

pub type TempoProvider = FillProvider<
    JoinFill<Identity, TempoFillers<NonceFiller>>,
    RootProvider<TempoNetwork>,
    TempoNetwork,
>;

pub async fn connect_tempo_url(url: &str) -> Result<TempoProvider, ChainAccessError> {
    ProviderBuilder::new_with_network::<TempoNetwork>()
        .connect(url)
        .await
        .map_err(|e| ChainAccessError::Rpc(format!("failed to connect to Tempo RPC at {url}: {e}")))
}
