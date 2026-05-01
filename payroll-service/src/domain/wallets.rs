use alloy_primitives::Address;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WalletAddress(String);

impl WalletAddress {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, WalletAddressError> {
        let raw = raw.as_ref().trim();

        if raw.is_empty() {
            return Err(WalletAddressError::Empty);
        }

        if !raw.starts_with("0x") {
            return Err(WalletAddressError::MissingPrefix);
        }

        let address = if is_mixed_case(raw) {
            Address::parse_checksummed(raw, None)
                .map_err(|_| WalletAddressError::InvalidChecksum)?
        } else {
            Address::from_str(raw).map_err(|_| WalletAddressError::Invalid)?
        };

        Ok(Self(address.to_checksum(None)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
fn is_mixed_case(raw: &str) -> bool {
    let without_prefix = raw.strip_prefix("0x").unwrap_or(raw);
    let has_lower = without_prefix.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = without_prefix.chars().any(|c| c.is_ascii_uppercase());
    has_lower && has_upper
}

#[derive(Debug, thiserror::Error)]
pub enum WalletAddressError {
    #[error("wallet address is empty")]
    Empty,
    #[error("wallet address must start with 0x")]
    MissingPrefix,
    #[error("wallet address is not a valid EVM address")]
    Invalid,
    #[error("wallet address checksum is invalid")]
    InvalidChecksum,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_wallet_address() {
        assert_eq!(
            WalletAddress::parse("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            WalletAddress("0x1234567890AbcdEF1234567890aBcdef12345678".to_string())
        );
    }

    #[test]
    fn parses_valid_checksummed_wallet_address_with_mixed_case() {
        assert_eq!(
            WalletAddress::parse("0x1234567890AbcdEF1234567890aBcdef12345678").unwrap(),
            WalletAddress("0x1234567890AbcdEF1234567890aBcdef12345678".to_string())
        );
    }

    #[test]
    fn fails_on_invalid_mixed_case_checksum() {
        assert!(matches!(
            WalletAddress::parse("0x1234567890AbcDef1234567890AbcDef12345678"),
            Err(WalletAddressError::InvalidChecksum)
        ));
    }

    #[test]
    fn fails_on_empty_address() {
        assert!(WalletAddress::parse("").is_err());
    }

    #[test]
    fn fails_on_invalid_address() {
        assert!(WalletAddress::parse("invalid-address").is_err());
    }
}
