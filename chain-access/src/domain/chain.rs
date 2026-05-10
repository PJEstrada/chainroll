use crate::domain::network::Network;
use clap::ValueEnum;

/// Supported chains for this library.
/// This enum is part of the public API and should remain small and stable.
/// Chain-specific behavior lives behind adapter implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum Chain {
    Arc,
    Tempo,
}

impl Chain {
    /// Canonical identifier used in configs and CLI flags.
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Arc => "arc",
            Chain::Tempo => "tempo",
        }
    }
}

impl core::fmt::Display for Chain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for Chain {
    type Err = ParseChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "arc" => Ok(Chain::Arc),
            "tempo" => Ok(Chain::Tempo),
            other => Err(ParseChainError {
                input: other.to_string(),
            }),
        }
    }
}

/// RPC connection configuration for a single chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcConfig {
    pub network: Network,
    pub chain: Chain,
}

impl RpcConfig {
    pub fn new(network: Network, chain: Chain) -> Self {
        Self { network, chain }
    }

    pub fn url(&self) -> String {
        match (self.chain, self.network) {
            (Chain::Tempo, Network::Testnet) => "https://rpc.moderato.tempo.xyz".to_string(),

            (Chain::Arc, Network::Testnet) => "https://arc-testnet.circle.com".to_string(),

            // Fail fast for unsupported combos
            (Chain::Tempo, Network::Mainnet) => {
                panic!("Tempo mainnet not supported yet")
            }
            (Chain::Arc, Network::Mainnet) => {
                panic!("Arc mainnet not supported yet")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseChainError {
    pub input: String,
}

impl core::fmt::Display for ParseChainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unknown chain '{}'", self.input)
    }
}

impl std::error::Error for ParseChainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_to_string() {
        assert_eq!(Chain::Arc.to_string(), "arc");
        assert_eq!(Chain::Tempo.to_string(), "tempo");
    }

    #[test]
    fn chain_from_str() {
        assert_eq!("arc".parse::<Chain>().unwrap(), Chain::Arc);
        assert_eq!("ARC".parse::<Chain>().unwrap(), Chain::Arc);
        assert_eq!(" tempo ".parse::<Chain>().unwrap(), Chain::Tempo);
    }

    #[test]
    fn chain_from_str_invalid() {
        let err = "arbitrum".parse::<Chain>().unwrap_err();
        assert_eq!(err.to_string(), "unknown chain 'arbitrum'");
    }

    #[test]
    fn rpc_config_new() {
        let cfg = RpcConfig::new(Network::Testnet, Chain::Arc);
        assert_eq!(cfg.network, Network::Testnet);
        assert_eq!(cfg.chain, Chain::Arc);
    }
}
