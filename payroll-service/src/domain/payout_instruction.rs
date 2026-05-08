use crate::domain::compensation::CompensationAmount;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::payrun::{IDPayrun, IDPayrunItem, Payrun, PayrunItem, PayrunItemStatus};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{
    IDTreasuryAccount, TokenSymbol, TreasuryAccount, TreasuryChain, TreasuryControlMode,
    TreasuryCustodyProvider, validate_token_decimals,
};
use crate::domain::wallets::WalletAddress;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const MAX_IDEMPOTENCY_KEY_LEN: usize = 200;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDPayoutInstruction;

impl IDResource for IDPayoutInstruction {
    fn prefix() -> Option<String> {
        Some("payout_instruction".to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PayoutInstruction {
    id: StandardID<IDPayoutInstruction>,
    tenant_id: StandardID<IDTenant>,
    payrun_id: StandardID<IDPayrun>,
    payrun_item_id: StandardID<IDPayrunItem>,
    employee_id: StandardID<IDEmployee>,
    treasury_account_id: StandardID<IDTreasuryAccount>,
    idempotency_key: PayoutInstructionIdempotencyKey,
    destination_wallet_address: WalletAddress,
    source_wallet_address: WalletAddress,
    chain: TreasuryChain,
    chain_id: u64,
    token_symbol: TokenSymbol,
    token_address: WalletAddress,
    token_decimals: u8,
    amount: CompensationAmount,
    custody_provider: TreasuryCustodyProvider,
    control_mode: TreasuryControlMode,
    provider_wallet_id: Option<String>,
    provider_owner_id: Option<String>,
    secret_reference: Option<String>,
    created_at: DateTime<Utc>,
}

impl PayoutInstruction {
    pub fn new(
        payrun: &Payrun,
        item: &PayrunItem,
        employee: &Employee,
        treasury_account: &TreasuryAccount,
    ) -> Result<Self, PayoutInstructionError> {
        if item.status() != PayrunItemStatus::Payable {
            return Err(PayoutInstructionError::PayrunItemMustBePayable);
        }

        let amount = item
            .amount()
            .cloned()
            .ok_or(PayoutInstructionError::MissingAmount)?;
        let destination_wallet_address = employee
            .wallet_address()
            .clone()
            .ok_or(PayoutInstructionError::MissingEmployeeWallet)?;

        if payrun.tenant_id() != treasury_account.tenant_id() {
            return Err(PayoutInstructionError::TreasuryTenantMismatch);
        }

        if item.employee_id() != employee.id() {
            return Err(PayoutInstructionError::EmployeeMismatch);
        }

        if amount.token_symbol() != treasury_account.token_symbol() {
            return Err(PayoutInstructionError::TreasuryTokenMismatch);
        }

        if !treasury_account.can_auto_submit() {
            return Err(PayoutInstructionError::TreasuryCannotAutoSubmit);
        }

        Self::restore(
            StandardID::new(),
            PayoutInstructionDraft {
                tenant_id: *payrun.tenant_id(),
                payrun_id: *payrun.id(),
                payrun_item_id: *item.id(),
                employee_id: *employee.id(),
                treasury_account_id: *treasury_account.id(),
                idempotency_key: PayoutInstructionIdempotencyKey::for_payrun_item(item.id())?,
                destination_wallet_address,
                source_wallet_address: treasury_account.sender_address().clone(),
                chain: treasury_account.chain(),
                chain_id: treasury_account.chain_id(),
                token_symbol: treasury_account.token_symbol().clone(),
                token_address: treasury_account.token_address().clone(),
                token_decimals: treasury_account.token_decimals(),
                amount,
                custody_provider: treasury_account.custody_provider(),
                control_mode: treasury_account.control_mode(),
                provider_wallet_id: treasury_account.provider_wallet_id().map(str::to_string),
                provider_owner_id: treasury_account.provider_owner_id().map(str::to_string),
                secret_reference: treasury_account.secret_reference().map(str::to_string),
            },
            Utc::now(),
        )
    }

    pub fn restore(
        id: StandardID<IDPayoutInstruction>,
        mut draft: PayoutInstructionDraft,
        created_at: DateTime<Utc>,
    ) -> Result<Self, PayoutInstructionError> {
        draft.normalize();
        validate_token_decimals(draft.token_decimals)?;
        validate_chain_id(draft.chain, draft.chain_id)?;
        validate_amount_token(&draft.amount, &draft.token_symbol)?;
        validate_auto_submit_control(draft.control_mode)?;

        Ok(Self {
            id,
            tenant_id: draft.tenant_id,
            payrun_id: draft.payrun_id,
            payrun_item_id: draft.payrun_item_id,
            employee_id: draft.employee_id,
            treasury_account_id: draft.treasury_account_id,
            idempotency_key: draft.idempotency_key,
            destination_wallet_address: draft.destination_wallet_address,
            source_wallet_address: draft.source_wallet_address,
            chain: draft.chain,
            chain_id: draft.chain_id,
            token_symbol: draft.token_symbol,
            token_address: draft.token_address,
            token_decimals: draft.token_decimals,
            amount: draft.amount,
            custody_provider: draft.custody_provider,
            control_mode: draft.control_mode,
            provider_wallet_id: draft.provider_wallet_id,
            provider_owner_id: draft.provider_owner_id,
            secret_reference: draft.secret_reference,
            created_at,
        })
    }

    pub fn id(&self) -> &StandardID<IDPayoutInstruction> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn payrun_id(&self) -> &StandardID<IDPayrun> {
        &self.payrun_id
    }

    pub fn payrun_item_id(&self) -> &StandardID<IDPayrunItem> {
        &self.payrun_item_id
    }

    pub fn employee_id(&self) -> &StandardID<IDEmployee> {
        &self.employee_id
    }

    pub fn treasury_account_id(&self) -> &StandardID<IDTreasuryAccount> {
        &self.treasury_account_id
    }

    pub fn idempotency_key(&self) -> &PayoutInstructionIdempotencyKey {
        &self.idempotency_key
    }

    pub fn destination_wallet_address(&self) -> &WalletAddress {
        &self.destination_wallet_address
    }

    pub fn source_wallet_address(&self) -> &WalletAddress {
        &self.source_wallet_address
    }

    pub fn chain(&self) -> TreasuryChain {
        self.chain
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
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

    pub fn amount(&self) -> &CompensationAmount {
        &self.amount
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Clone)]
pub struct PayoutInstructionDraft {
    pub tenant_id: StandardID<IDTenant>,
    pub payrun_id: StandardID<IDPayrun>,
    pub payrun_item_id: StandardID<IDPayrunItem>,
    pub employee_id: StandardID<IDEmployee>,
    pub treasury_account_id: StandardID<IDTreasuryAccount>,
    pub idempotency_key: PayoutInstructionIdempotencyKey,
    pub destination_wallet_address: WalletAddress,
    pub source_wallet_address: WalletAddress,
    pub chain: TreasuryChain,
    pub chain_id: u64,
    pub token_symbol: TokenSymbol,
    pub token_address: WalletAddress,
    pub token_decimals: u8,
    pub amount: CompensationAmount,
    pub custody_provider: TreasuryCustodyProvider,
    pub control_mode: TreasuryControlMode,
    pub provider_wallet_id: Option<String>,
    pub provider_owner_id: Option<String>,
    pub secret_reference: Option<String>,
}

impl PayoutInstructionDraft {
    fn normalize(&mut self) {
        self.provider_wallet_id = normalize_optional_string(self.provider_wallet_id.take());
        self.provider_owner_id = normalize_optional_string(self.provider_owner_id.take());
        self.secret_reference = normalize_optional_string(self.secret_reference.take());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PayoutInstructionIdempotencyKey(String);

impl PayoutInstructionIdempotencyKey {
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, PayoutInstructionError> {
        let raw = raw.as_ref().trim();
        if raw.is_empty() {
            return Err(PayoutInstructionError::EmptyIdempotencyKey);
        }
        if raw.len() > MAX_IDEMPOTENCY_KEY_LEN {
            return Err(PayoutInstructionError::IdempotencyKeyTooLong {
                max: MAX_IDEMPOTENCY_KEY_LEN,
            });
        }

        Ok(Self(raw.to_string()))
    }

    pub fn for_payrun_item(
        payrun_item_id: &StandardID<IDPayrunItem>,
    ) -> Result<Self, PayoutInstructionError> {
        Self::parse(format!("payrun_item:{payrun_item_id}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PayoutInstructionError {
    #[error("payrun item must be payable")]
    PayrunItemMustBePayable,
    #[error("payable payrun item is missing an amount")]
    MissingAmount,
    #[error("payable employee is missing a wallet")]
    MissingEmployeeWallet,
    #[error("employee does not match payrun item")]
    EmployeeMismatch,
    #[error("treasury account belongs to a different tenant")]
    TreasuryTenantMismatch,
    #[error("treasury account token does not match payrun item token")]
    TreasuryTokenMismatch,
    #[error("treasury account cannot be submitted automatically")]
    TreasuryCannotAutoSubmit,
    #[error("invalid chain id: expected {expected}, got {actual}")]
    InvalidChainId { expected: u64, actual: u64 },
    #[error("idempotency key cannot be empty")]
    EmptyIdempotencyKey,
    #[error("idempotency key cannot be longer than {max} characters")]
    IdempotencyKeyTooLong { max: usize },
    #[error("invalid treasury account: {0}")]
    InvalidTreasuryAccount(#[from] crate::domain::treasury::TreasuryAccountError),
}

fn validate_chain_id(chain: TreasuryChain, chain_id: u64) -> Result<(), PayoutInstructionError> {
    if chain.chain_id() != chain_id {
        return Err(PayoutInstructionError::InvalidChainId {
            expected: chain.chain_id(),
            actual: chain_id,
        });
    }

    Ok(())
}

fn validate_amount_token(
    amount: &CompensationAmount,
    token_symbol: &TokenSymbol,
) -> Result<(), PayoutInstructionError> {
    if amount.token_symbol() != token_symbol {
        return Err(PayoutInstructionError::TreasuryTokenMismatch);
    }

    Ok(())
}

fn validate_auto_submit_control(
    control_mode: TreasuryControlMode,
) -> Result<(), PayoutInstructionError> {
    if !matches!(
        control_mode,
        TreasuryControlMode::ServerControlled | TreasuryControlMode::UserDelegated
    ) {
        return Err(PayoutInstructionError::TreasuryCannotAutoSubmit);
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
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::payrun::{PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{TreasuryAccountDraft, TreasuryChain};

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn amount(token_symbol: &str) -> CompensationAmount {
        CompensationAmount::new(1_000_000, TokenSymbol::parse(token_symbol).unwrap()).unwrap()
    }

    fn payrun(token_symbol: &str) -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    amount(token_symbol),
                )],
            ),
            crate::domain::payrun::CreatePayrunOptions::strict(),
        )
        .unwrap()
    }

    fn employee(id: StandardID<IDEmployee>, wallet_address: Option<WalletAddress>) -> Employee {
        Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(id)
            .with_wallet_address(wallet_address)
    }

    fn treasury_account(tenant_id: StandardID<IDTenant>, token_symbol: &str) -> TreasuryAccount {
        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id,
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse(token_symbol).unwrap(),
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
        .unwrap()
    }

    #[test]
    fn creates_instruction_from_payable_payrun_item() {
        let payrun = payrun("USDC");
        let item = &payrun.items()[0];
        let employee = employee(
            *item.employee_id(),
            Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")),
        );
        let treasury = treasury_account(*payrun.tenant_id(), "USDC");

        let instruction = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap();

        assert_eq!(instruction.payrun_id(), payrun.id());
        assert_eq!(instruction.payrun_item_id(), item.id());
        assert_eq!(instruction.employee_id(), employee.id());
        assert_eq!(instruction.treasury_account_id(), treasury.id());
        assert_eq!(
            instruction.idempotency_key().as_str(),
            format!("payrun_item:{}", item.id())
        );
        assert_eq!(instruction.amount().amount_units(), 1_000_000);
        assert_eq!(instruction.token_symbol().as_str(), "USDC");
    }

    #[test]
    fn rejects_employee_without_wallet() {
        let payrun = payrun("USDC");
        let item = &payrun.items()[0];
        let employee = employee(*item.employee_id(), None);
        let treasury = treasury_account(*payrun.tenant_id(), "USDC");

        let err = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap_err();

        assert!(matches!(err, PayoutInstructionError::MissingEmployeeWallet));
    }

    #[test]
    fn rejects_treasury_token_mismatch() {
        let payrun = payrun("USDC");
        let item = &payrun.items()[0];
        let employee = employee(
            *item.employee_id(),
            Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")),
        );
        let treasury = treasury_account(*payrun.tenant_id(), "pathUSD");

        let err = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap_err();

        assert!(matches!(err, PayoutInstructionError::TreasuryTokenMismatch));
    }

    #[test]
    fn validates_idempotency_key() {
        assert!(PayoutInstructionIdempotencyKey::parse("").is_err());
        assert!(PayoutInstructionIdempotencyKey::parse("payrun_item:abc").is_ok());
    }
}
