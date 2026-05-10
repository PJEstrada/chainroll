use crate::domain::chain_info::ChainInfo;

pub struct TempoTestnet;

impl ChainInfo for TempoTestnet {
    fn chain_id(&self) -> u64 {
        42431
    }

    fn name(&self) -> &'static str {
        "tempo-testnet"
    }

    fn rpc_url(&self) -> &'static str {
        "https://rpc.moderato.tempo.xyz"
    }

    fn explorer_url(&self) -> &'static str {
        "https://explore.tempo.xyz"
    }
}

pub static CONFIG: TempoTestnet = TempoTestnet;
