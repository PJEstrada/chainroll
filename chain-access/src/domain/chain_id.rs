use crate::domain::chain_info::ChainInfo;
use crate::domain::chains;

/// Discriminant for selecting a chain.
/// All chain metadata lives in the corresponding `ChainInfo` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainId {
    TempoTestnet,
}

impl ChainId {
    /// Returns the static `ChainInfo` for this chain.
    pub fn info(self) -> &'static dyn ChainInfo {
        match self {
            ChainId::TempoTestnet => &chains::tempo_testnet::CONFIG,
        }
    }
}

impl core::fmt::Display for ChainId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.info().name())
    }
}

impl core::str::FromStr for ChainId {
    type Err = ParseChainIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tempo-testnet" | "tempo_testnet" => Ok(ChainId::TempoTestnet),
            other => Err(ParseChainIdError {
                input: other.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseChainIdError {
    pub input: String,
}

impl core::fmt::Display for ParseChainIdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unknown chain '{}' (valid: tempo-testnet)", self.input)
    }
}

impl std::error::Error for ParseChainIdError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_from_str() {
        assert_eq!(
            "tempo-testnet".parse::<ChainId>().unwrap(),
            ChainId::TempoTestnet
        );
        assert_eq!(
            "tempo_testnet".parse::<ChainId>().unwrap(),
            ChainId::TempoTestnet
        );
    }

    #[test]
    fn chain_id_from_str_invalid() {
        let err = "mainnet".parse::<ChainId>().unwrap_err();
        assert!(err.to_string().contains("mainnet"));
    }

    #[test]
    fn chain_id_display() {
        assert_eq!(ChainId::TempoTestnet.to_string(), "tempo-testnet");
    }

    #[test]
    fn chain_id_info() {
        let cfg = ChainId::TempoTestnet.info();
        assert_eq!(cfg.chain_id(), 42431);
        assert_eq!(cfg.name(), "tempo-testnet");
        assert_eq!(cfg.tx_url("0xabc"), "https://explore.tempo.xyz/tx/0xabc");
        assert_eq!(
            cfg.address_url("0xdef"),
            "https://explore.tempo.xyz/address/0xdef"
        );
    }
}
