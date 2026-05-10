use crate::domain::ids::StandardID;
use crate::domain::payout_instruction::{
    IDPayoutInstruction, PayoutInstruction, PayoutInstructionIdempotencyKey,
};
use crate::domain::treasury::{
    TokenSymbol, TreasuryChain, TreasuryControlMode, TreasuryCustodyProvider,
};
use crate::domain::wallets::WalletAddress;
use serde::Serialize;

const MAX_PROVIDER_REFERENCE_LEN: usize = 200;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait StablecoinPayoutClient: Send + Sync + 'static {
    async fn submit_payout(
        &self,
        request: StablecoinPayoutRequest,
    ) -> Result<StablecoinPayoutOutcome, StablecoinPayoutClientError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StablecoinPayoutRequest {
    instruction_id: StandardID<IDPayoutInstruction>,
    idempotency_key: PayoutInstructionIdempotencyKey,
    chain: TreasuryChain,
    chain_id: u64,
    source_wallet_address: WalletAddress,
    destination_wallet_address: WalletAddress,
    token_symbol: TokenSymbol,
    token_address: WalletAddress,
    token_decimals: u8,
    amount_units: u128,
    custody_provider: TreasuryCustodyProvider,
    control_mode: TreasuryControlMode,
    provider_wallet_id: Option<String>,
    provider_owner_id: Option<String>,
    secret_reference: Option<String>,
}

impl StablecoinPayoutRequest {
    pub fn from_instruction(instruction: &PayoutInstruction) -> Self {
        Self {
            instruction_id: *instruction.id(),
            idempotency_key: instruction.idempotency_key().clone(),
            chain: instruction.chain(),
            chain_id: instruction.chain_id(),
            source_wallet_address: instruction.source_wallet_address().clone(),
            destination_wallet_address: instruction.destination_wallet_address().clone(),
            token_symbol: instruction.token_symbol().clone(),
            token_address: instruction.token_address().clone(),
            token_decimals: instruction.token_decimals(),
            amount_units: instruction.amount().amount_units(),
            custody_provider: instruction.custody_provider(),
            control_mode: instruction.control_mode(),
            provider_wallet_id: instruction.provider_wallet_id().map(str::to_string),
            provider_owner_id: instruction.provider_owner_id().map(str::to_string),
            secret_reference: instruction.secret_reference().map(str::to_string),
        }
    }

    pub fn instruction_id(&self) -> &StandardID<IDPayoutInstruction> {
        &self.instruction_id
    }

    pub fn idempotency_key(&self) -> &PayoutInstructionIdempotencyKey {
        &self.idempotency_key
    }

    pub fn chain(&self) -> TreasuryChain {
        self.chain
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub fn source_wallet_address(&self) -> &WalletAddress {
        &self.source_wallet_address
    }

    pub fn destination_wallet_address(&self) -> &WalletAddress {
        &self.destination_wallet_address
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

    pub fn amount_units(&self) -> u128 {
        self.amount_units
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

    #[cfg(test)]
    pub(crate) fn without_provider_wallet_id(mut self) -> Self {
        self.provider_wallet_id = None;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StablecoinPayoutOutcome {
    Submitted(SubmittedStablecoinPayout),
    Rejected(RejectedStablecoinPayout),
    ReviewRequired(ReviewRequiredStablecoinPayout),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubmittedStablecoinPayout {
    provider_reference: ProviderPayoutReference,
    transaction_hash: Option<TransactionHash>,
}

impl SubmittedStablecoinPayout {
    pub fn new(
        provider_reference: ProviderPayoutReference,
        transaction_hash: Option<TransactionHash>,
    ) -> Self {
        Self {
            provider_reference,
            transaction_hash,
        }
    }

    pub fn provider_reference(&self) -> &ProviderPayoutReference {
        &self.provider_reference
    }

    pub fn transaction_hash(&self) -> Option<&TransactionHash> {
        self.transaction_hash.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RejectedStablecoinPayout {
    reason: StablecoinPayoutRejectionReason,
    provider_reference: Option<ProviderPayoutReference>,
}

impl RejectedStablecoinPayout {
    pub fn new(
        reason: StablecoinPayoutRejectionReason,
        provider_reference: Option<ProviderPayoutReference>,
    ) -> Self {
        Self {
            reason,
            provider_reference,
        }
    }

    pub fn reason(&self) -> &StablecoinPayoutRejectionReason {
        &self.reason
    }

    pub fn provider_reference(&self) -> Option<&ProviderPayoutReference> {
        self.provider_reference.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewRequiredStablecoinPayout {
    reason: StablecoinPayoutReviewReason,
    provider_reference: Option<ProviderPayoutReference>,
}

impl ReviewRequiredStablecoinPayout {
    pub fn new(
        reason: StablecoinPayoutReviewReason,
        provider_reference: Option<ProviderPayoutReference>,
    ) -> Self {
        Self {
            reason,
            provider_reference,
        }
    }

    pub fn reason(&self) -> &StablecoinPayoutReviewReason {
        &self.reason
    }

    pub fn provider_reference(&self) -> Option<&ProviderPayoutReference> {
        self.provider_reference.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StablecoinPayoutRejectionReason {
    InsufficientFunds,
    InvalidSourceWallet,
    InvalidDestinationWallet,
    UnsupportedChain,
    UnsupportedCustodyProvider,
    UnsupportedControlMode,
    UnsupportedToken,
    ProviderRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StablecoinPayoutReviewReason {
    AmbiguousProviderResponse,
    TimeoutAfterSubmission,
    ProviderStatusUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ProviderPayoutReference(String);

impl ProviderPayoutReference {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, StablecoinPayoutValueError> {
        let raw = raw.as_ref().trim();
        if raw.is_empty() {
            return Err(StablecoinPayoutValueError::EmptyProviderReference);
        }
        if raw.len() > MAX_PROVIDER_REFERENCE_LEN {
            return Err(StablecoinPayoutValueError::ProviderReferenceTooLong {
                max: MAX_PROVIDER_REFERENCE_LEN,
            });
        }

        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TransactionHash(String);

impl TransactionHash {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, StablecoinPayoutValueError> {
        let raw = raw.as_ref().trim();
        let hash = raw
            .strip_prefix("0x")
            .ok_or(StablecoinPayoutValueError::InvalidTransactionHash)?;

        if hash.len() != 64 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(StablecoinPayoutValueError::InvalidTransactionHash);
        }

        Ok(Self(format!("0x{}", hash.to_ascii_lowercase())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StablecoinPayoutValueError {
    #[error("provider reference cannot be empty")]
    EmptyProviderReference,
    #[error("provider reference cannot be longer than {max} characters")]
    ProviderReferenceTooLong { max: usize },
    #[error("transaction hash must be a 0x-prefixed 32-byte hex value")]
    InvalidTransactionHash,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StablecoinPayoutClientError {
    #[error("stablecoin payout client is misconfigured: {0}")]
    Configuration(String),
    #[error("stablecoin payout provider is unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("stablecoin payout transport error: {0}")]
    Transport(String),
    #[error("stablecoin payout serialization error: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::Employee;
    use crate::domain::payout_instruction::PayoutInstruction;
    use crate::domain::payrun::{CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{TreasuryAccount, TreasuryAccountDraft, TreasuryCustodyProvider};
    use crate::domain::treasury::{TreasuryChain, TreasuryControlMode};

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn instruction() -> PayoutInstruction {
        let payrun = Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                        .unwrap(),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap();
        let item = &payrun.items()[0];
        let employee = Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(*item.employee_id())
            .with_wallet_address(Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")));
        let treasury = TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id: *payrun.tenant_id(),
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("USDC").unwrap(),
            token_address: wallet("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: wallet("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        })
        .unwrap();

        PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap()
    }

    #[test]
    fn builds_request_from_instruction_snapshot() {
        let instruction = instruction();

        let request = StablecoinPayoutRequest::from_instruction(&instruction);

        assert_eq!(request.instruction_id(), instruction.id());
        assert_eq!(request.idempotency_key(), instruction.idempotency_key());
        assert_eq!(request.amount_units(), 1_000_000);
        assert_eq!(request.token_symbol().as_str(), "USDC");
        assert_eq!(request.chain_id(), 42431);
        assert_eq!(
            request.secret_reference(),
            Some("env:TEMPO_TREASURY_PRIVATE_KEY")
        );
    }

    #[test]
    fn validates_provider_reference() {
        assert!(ProviderPayoutReference::parse("").is_err());
        assert_eq!(
            ProviderPayoutReference::parse(" provider-123 ")
                .unwrap()
                .as_str(),
            "provider-123"
        );
    }

    #[test]
    fn validates_transaction_hash() {
        let hash = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        assert_eq!(TransactionHash::parse(hash).unwrap().as_str(), hash);
        assert!(TransactionHash::parse("0xabc").is_err());
        assert!(TransactionHash::parse("abc").is_err());
    }
}
