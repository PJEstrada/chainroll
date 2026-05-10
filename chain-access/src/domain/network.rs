use crate::domain::chain::Chain;
use crate::domain::chain_ids::chains::TEMPO_TESTNET;
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

pub fn chain_id(chain: Chain, network: Network) -> anyhow::Result<u64> {
    match (chain, network) {
        (Chain::Tempo, Network::Mainnet) => Ok(TEMPO_TESTNET),
        _ => anyhow::bail!("Unsupported chain type"),
    }
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        }
    }
}

impl core::fmt::Display for Network {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseNetworkError {
    pub input: String,
}

impl core::str::FromStr for Network {
    type Err = ParseNetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            other => Err(ParseNetworkError {
                input: other.to_string(),
            }),
        }
    }
}
