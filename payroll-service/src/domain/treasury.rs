use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus};
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::query::Query;
use crate::domain::tenant::IDTenant;
use crate::domain::wallets::WalletAddress;
use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Display;
use std::str::FromStr;

const MAX_TREASURY_NAME_LEN: usize = 100;
const MAX_TOKEN_SYMBOL_LEN: usize = 32;
const MAX_TOKEN_DECIMALS: u8 = 36;

#[derive(Debug, Clone, Serialize)]
pub struct TreasuryAccount {
    id: StandardID<IDTreasuryAccount>,
    tenant_id: StandardID<IDTenant>,
    metadata: LifecycleMeta,
    name: String,
    chain: TreasuryChain,
    token_symbol: TokenSymbol,
    token_address: WalletAddress,
    token_decimals: u8,
    sender_address: WalletAddress,
    custody_provider: TreasuryCustodyProvider,
    control_mode: TreasuryControlMode,
    provider_wallet_id: Option<String>,
    provider_owner_id: Option<String>,
    secret_reference: Option<String>,
    is_default: bool,
}

impl TreasuryAccount {
    pub fn new(draft: TreasuryAccountDraft) -> Result<Self, TreasuryAccountError> {
        let now = Utc::now();
        Self::restore(
            StandardID::new(),
            LifecycleMeta {
                status: ObjectStatus::Active,
                created: now,
                updated: now,
            },
            draft,
        )
    }

    pub fn restore(
        id: StandardID<IDTreasuryAccount>,
        metadata: LifecycleMeta,
        mut draft: TreasuryAccountDraft,
    ) -> Result<Self, TreasuryAccountError> {
        draft.normalize();
        validate_name(&draft.name)?;
        validate_token_decimals(draft.token_decimals)?;
        validate_default_status(metadata.status, draft.is_default)?;
        validate_control_configuration(
            draft.custody_provider,
            draft.control_mode,
            draft.provider_wallet_id.as_deref(),
            draft.provider_owner_id.as_deref(),
            draft.secret_reference.as_deref(),
        )?;

        Ok(Self {
            id,
            tenant_id: draft.tenant_id,
            metadata,
            name: draft.name,
            chain: draft.chain,
            token_symbol: draft.token_symbol,
            token_address: draft.token_address,
            token_decimals: draft.token_decimals,
            sender_address: draft.sender_address,
            custody_provider: draft.custody_provider,
            control_mode: draft.control_mode,
            provider_wallet_id: draft.provider_wallet_id,
            provider_owner_id: draft.provider_owner_id,
            secret_reference: draft.secret_reference,
            is_default: draft.is_default,
        })
    }

    pub fn id(&self) -> &StandardID<IDTreasuryAccount> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn metadata(&self) -> &LifecycleMeta {
        &self.metadata
    }

    pub fn status(&self) -> ObjectStatus {
        self.metadata.status
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn chain(&self) -> TreasuryChain {
        self.chain
    }

    pub fn chain_id(&self) -> u64 {
        self.chain.chain_id()
    }

    pub fn token_symbol(&self) -> &TokenSymbol {
        &self.token_symbol
    }

    pub fn token_address(&self) -> &WalletAddress {
        &self.token_address
    }

    pub fn token_decimals(&self) -> u8 {
        self.token_decimals
    }

    pub fn sender_address(&self) -> &WalletAddress {
        &self.sender_address
    }

    pub fn custody_provider(&self) -> TreasuryCustodyProvider {
        self.custody_provider
    }

    pub fn control_mode(&self) -> TreasuryControlMode {
        self.control_mode
    }

    pub fn provider_wallet_id(&self) -> Option<&str> {
        self.provider_wallet_id.as_deref()
    }

    pub fn provider_owner_id(&self) -> Option<&str> {
        self.provider_owner_id.as_deref()
    }

    pub fn secret_reference(&self) -> Option<&str> {
        self.secret_reference.as_deref()
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn can_auto_submit(&self) -> bool {
        matches!(
            self.control_mode,
            TreasuryControlMode::ServerControlled | TreasuryControlMode::UserDelegated
        )
    }

    pub fn requires_user_signature(&self) -> bool {
        matches!(
            self.control_mode,
            TreasuryControlMode::UserSignatureRequired
        )
    }

    pub fn mark_default(&mut self) -> Result<(), TreasuryAccountError> {
        validate_default_status(self.metadata.status, true)?;
        self.is_default = true;
        self.touch();
        Ok(())
    }

    pub fn clear_default(&mut self) {
        self.is_default = false;
        self.touch();
    }

    pub fn activate(&mut self) {
        self.metadata.status = ObjectStatus::Active;
        self.touch();
    }

    pub fn deactivate(&mut self) {
        self.metadata.status = ObjectStatus::Inactive;
        self.is_default = false;
        self.touch();
    }

    fn touch(&mut self) {
        self.metadata.updated = Utc::now();
    }
}

#[derive(Debug, Clone)]
pub struct TreasuryAccountDraft {
    pub tenant_id: StandardID<IDTenant>,
    pub name: String,
    pub chain: TreasuryChain,
    pub token_symbol: TokenSymbol,
    pub token_address: WalletAddress,
    pub token_decimals: u8,
    pub sender_address: WalletAddress,
    pub custody_provider: TreasuryCustodyProvider,
    pub control_mode: TreasuryControlMode,
    pub provider_wallet_id: Option<String>,
    pub provider_owner_id: Option<String>,
    pub secret_reference: Option<String>,
    pub is_default: bool,
}

impl TreasuryAccountDraft {
    fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.provider_wallet_id = normalize_optional_string(self.provider_wallet_id.take());
        self.provider_owner_id = normalize_optional_string(self.provider_owner_id.take());
        self.secret_reference = normalize_optional_string(self.secret_reference.take());
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreasuryAccountQuery {
    pub base: Query,
    pub status: Option<ObjectStatus>,
    pub chain: Option<TreasuryChain>,
    pub only_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TokenSymbol(String);

impl TokenSymbol {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, TreasuryAccountError> {
        let raw = raw.as_ref().trim();

        if raw.is_empty() {
            return Err(TreasuryAccountError::EmptyTokenSymbol);
        }

        if raw.len() > MAX_TOKEN_SYMBOL_LEN {
            return Err(TreasuryAccountError::TokenSymbolTooLong {
                max: MAX_TOKEN_SYMBOL_LEN,
            });
        }

        let is_valid = raw
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'));
        if !is_valid {
            return Err(TreasuryAccountError::InvalidTokenSymbol);
        }

        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for TokenSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for TokenSymbol {
    type Err = TreasuryAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<'de> Deserialize<'de> for TokenSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TreasuryChain {
    TempoTestnet,
}

impl TreasuryChain {
    pub fn chain_id(self) -> u64 {
        match self {
            TreasuryChain::TempoTestnet => 42431,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TreasuryChain::TempoTestnet => "tempo-testnet",
        }
    }

    pub fn explorer_url(self) -> &'static str {
        match self {
            TreasuryChain::TempoTestnet => "https://explore.tempo.xyz",
        }
    }

    pub fn rpc_url(self) -> &'static str {
        match self {
            TreasuryChain::TempoTestnet => "https://rpc.moderato.tempo.xyz",
        }
    }
}

impl Display for TreasuryChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TreasuryChain {
    type Err = ParseTreasuryChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tempo-testnet" | "tempo_testnet" => Ok(Self::TempoTestnet),
            other => Err(ParseTreasuryChainError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid treasury chain: {0}")]
pub struct ParseTreasuryChainError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryCustodyProvider {
    LocalKey,
    Privy,
    External,
}

impl TreasuryCustodyProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalKey => "local_key",
            Self::Privy => "privy",
            Self::External => "external",
        }
    }
}

impl Display for TreasuryCustodyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TreasuryCustodyProvider {
    type Err = ParseTreasuryCustodyProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "local_key" | "local-key" => Ok(Self::LocalKey),
            "privy" => Ok(Self::Privy),
            "external" => Ok(Self::External),
            other => Err(ParseTreasuryCustodyProviderError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid treasury custody provider: {0}")]
pub struct ParseTreasuryCustodyProviderError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryControlMode {
    ServerControlled,
    UserDelegated,
    UserSignatureRequired,
    ExternalExecution,
}

impl TreasuryControlMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ServerControlled => "server_controlled",
            Self::UserDelegated => "user_delegated",
            Self::UserSignatureRequired => "user_signature_required",
            Self::ExternalExecution => "external_execution",
        }
    }
}

impl Display for TreasuryControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TreasuryControlMode {
    type Err = ParseTreasuryControlModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "server_controlled" | "server-controlled" => Ok(Self::ServerControlled),
            "user_delegated" | "user-delegated" => Ok(Self::UserDelegated),
            "user_signature_required" | "user-signature-required" => {
                Ok(Self::UserSignatureRequired)
            }
            "external_execution" | "external-execution" => Ok(Self::ExternalExecution),
            other => Err(ParseTreasuryControlModeError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid treasury control mode: {0}")]
pub struct ParseTreasuryControlModeError(String);

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDTreasuryAccount;

impl IDResource for IDTreasuryAccount {
    fn prefix() -> Option<String> {
        Some("treasury_account".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TreasuryAccountError {
    #[error("treasury account name cannot be empty")]
    EmptyName,
    #[error("treasury account name cannot be longer than {max} characters")]
    NameTooLong { max: usize },
    #[error("token symbol cannot be empty")]
    EmptyTokenSymbol,
    #[error("token symbol cannot be longer than {max} characters")]
    TokenSymbolTooLong { max: usize },
    #[error("token symbol contains unsupported characters")]
    InvalidTokenSymbol,
    #[error("token decimals cannot be greater than {max}")]
    TokenDecimalsTooLarge { max: u8 },
    #[error("default treasury accounts must be active")]
    DefaultRequiresActiveAccount,
    #[error("secret reference is required for local key treasury accounts")]
    MissingSecretReference,
    #[error("provider wallet id is required for Privy treasury accounts")]
    MissingProviderWalletId,
    #[error("provider owner id is required for delegated Privy treasury accounts")]
    MissingProviderOwnerId,
    #[error("{provider} does not support {control_mode} control")]
    UnsupportedControlMode {
        provider: TreasuryCustodyProvider,
        control_mode: TreasuryControlMode,
    },
}

fn validate_name(name: &str) -> Result<(), TreasuryAccountError> {
    if name.is_empty() {
        return Err(TreasuryAccountError::EmptyName);
    }

    if name.len() > MAX_TREASURY_NAME_LEN {
        return Err(TreasuryAccountError::NameTooLong {
            max: MAX_TREASURY_NAME_LEN,
        });
    }

    Ok(())
}

fn validate_token_decimals(token_decimals: u8) -> Result<(), TreasuryAccountError> {
    if token_decimals > MAX_TOKEN_DECIMALS {
        return Err(TreasuryAccountError::TokenDecimalsTooLarge {
            max: MAX_TOKEN_DECIMALS,
        });
    }

    Ok(())
}

fn validate_default_status(
    status: ObjectStatus,
    is_default: bool,
) -> Result<(), TreasuryAccountError> {
    if is_default && status != ObjectStatus::Active {
        return Err(TreasuryAccountError::DefaultRequiresActiveAccount);
    }

    Ok(())
}

fn validate_control_configuration(
    provider: TreasuryCustodyProvider,
    control_mode: TreasuryControlMode,
    provider_wallet_id: Option<&str>,
    provider_owner_id: Option<&str>,
    secret_reference: Option<&str>,
) -> Result<(), TreasuryAccountError> {
    match (provider, control_mode) {
        (TreasuryCustodyProvider::LocalKey, TreasuryControlMode::ServerControlled) => {
            if secret_reference.is_none() {
                return Err(TreasuryAccountError::MissingSecretReference);
            }
        }
        (TreasuryCustodyProvider::LocalKey, _) => {
            return Err(TreasuryAccountError::UnsupportedControlMode {
                provider,
                control_mode,
            });
        }
        (
            TreasuryCustodyProvider::Privy,
            TreasuryControlMode::ServerControlled | TreasuryControlMode::UserSignatureRequired,
        ) => {
            if provider_wallet_id.is_none() {
                return Err(TreasuryAccountError::MissingProviderWalletId);
            }
        }
        (TreasuryCustodyProvider::Privy, TreasuryControlMode::UserDelegated) => {
            if provider_wallet_id.is_none() {
                return Err(TreasuryAccountError::MissingProviderWalletId);
            }
            if provider_owner_id.is_none() {
                return Err(TreasuryAccountError::MissingProviderOwnerId);
            }
        }
        (TreasuryCustodyProvider::Privy, TreasuryControlMode::ExternalExecution) => {
            return Err(TreasuryAccountError::UnsupportedControlMode {
                provider,
                control_mode,
            });
        }
        (TreasuryCustodyProvider::External, TreasuryControlMode::ExternalExecution) => {}
        (TreasuryCustodyProvider::External, _) => {
            return Err(TreasuryAccountError::UnsupportedControlMode {
                provider,
                control_mode,
            });
        }
    }

    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn draft() -> TreasuryAccountDraft {
        TreasuryAccountDraft {
            tenant_id: StandardID::new(),
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("pathUSD").unwrap(),
            token_address: address("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: address("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        }
    }

    #[test]
    fn creates_local_key_treasury_account() {
        let account = TreasuryAccount::new(draft()).unwrap();

        assert_eq!(account.status(), ObjectStatus::Active);
        assert_eq!(account.chain_id(), 42431);
        assert_eq!(account.token_symbol().as_str(), "pathUSD");
        assert!(account.is_default());
        assert!(account.can_auto_submit());
        assert_eq!(
            account.secret_reference(),
            Some("env:TEMPO_TREASURY_PRIVATE_KEY")
        );
    }

    #[test]
    fn trims_name_and_optional_references() {
        let mut draft = draft();
        draft.name = "  Payroll treasury  ".to_string();
        draft.secret_reference = Some("  env:PRIVATE_KEY  ".to_string());

        let account = TreasuryAccount::new(draft).unwrap();

        assert_eq!(account.name(), "Payroll treasury");
        assert_eq!(account.secret_reference(), Some("env:PRIVATE_KEY"));
    }

    #[test]
    fn local_key_requires_secret_reference() {
        let mut draft = draft();
        draft.secret_reference = None;

        let err = TreasuryAccount::new(draft).unwrap_err();

        assert_eq!(err, TreasuryAccountError::MissingSecretReference);
    }

    #[test]
    fn privy_server_controlled_requires_provider_wallet_id() {
        let mut draft = draft();
        draft.custody_provider = TreasuryCustodyProvider::Privy;
        draft.secret_reference = None;

        let err = TreasuryAccount::new(draft).unwrap_err();

        assert_eq!(err, TreasuryAccountError::MissingProviderWalletId);
    }

    #[test]
    fn privy_delegated_requires_owner_id() {
        let mut draft = draft();
        draft.custody_provider = TreasuryCustodyProvider::Privy;
        draft.control_mode = TreasuryControlMode::UserDelegated;
        draft.provider_wallet_id = Some("wallet_123".to_string());
        draft.secret_reference = None;

        let err = TreasuryAccount::new(draft).unwrap_err();

        assert_eq!(err, TreasuryAccountError::MissingProviderOwnerId);
    }

    #[test]
    fn external_accounts_use_external_execution_only() {
        let mut draft = draft();
        draft.custody_provider = TreasuryCustodyProvider::External;
        draft.control_mode = TreasuryControlMode::ExternalExecution;
        draft.secret_reference = None;

        let account = TreasuryAccount::new(draft).unwrap();

        assert!(!account.can_auto_submit());
    }

    #[test]
    fn inactive_accounts_cannot_be_restored_as_default() {
        let metadata = LifecycleMeta {
            status: ObjectStatus::Inactive,
            created: Utc::now(),
            updated: Utc::now(),
        };

        let err = TreasuryAccount::restore(StandardID::new(), metadata, draft()).unwrap_err();

        assert_eq!(err, TreasuryAccountError::DefaultRequiresActiveAccount);
    }

    #[test]
    fn deactivating_clears_default_status() {
        let mut account = TreasuryAccount::new(draft()).unwrap();

        account.deactivate();

        assert_eq!(account.status(), ObjectStatus::Inactive);
        assert!(!account.is_default());
    }

    #[test]
    fn parses_treasury_enums_from_database_values() {
        assert_eq!(
            "tempo-testnet".parse::<TreasuryChain>().unwrap(),
            TreasuryChain::TempoTestnet
        );
        assert_eq!(
            "local_key".parse::<TreasuryCustodyProvider>().unwrap(),
            TreasuryCustodyProvider::LocalKey
        );
        assert_eq!(
            "user_delegated".parse::<TreasuryControlMode>().unwrap(),
            TreasuryControlMode::UserDelegated
        );
    }

    #[test]
    fn validates_token_symbol() {
        assert_eq!(TokenSymbol::parse("USDC.e").unwrap().as_str(), "USDC.e");
        assert_eq!(
            TokenSymbol::parse("bad symbol").unwrap_err(),
            TreasuryAccountError::InvalidTokenSymbol
        );
    }

    #[test]
    fn token_symbol_deserialization_uses_validation() {
        let err = serde_json::from_str::<TokenSymbol>("\"bad symbol\"").unwrap_err();

        assert!(err.to_string().contains("unsupported characters"));
    }
}
