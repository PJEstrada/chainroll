use crate::domain::audit::AuditEvent;
use crate::domain::compensation::{CompensationAmount, CompensationProfileError};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::{IdError, StandardID};
use crate::domain::payout_instruction::{
    IDPayoutInstruction, PayoutInstruction, PayoutInstructionDraft, PayoutInstructionError,
    PayoutInstructionIdempotencyKey,
};
use crate::domain::payrun::{IDPayrun, IDPayrunItem};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{
    IDTreasuryAccount, ParseTreasuryChainError, ParseTreasuryControlModeError,
    ParseTreasuryCustodyProviderError, TokenSymbol, TreasuryAccountError, TreasuryChain,
    TreasuryControlMode, TreasuryCustodyProvider,
};
use crate::domain::wallets::{WalletAddress, WalletAddressError};
use crate::services::datastore::PayoutInstructionStore;
use crate::services::datastore::postgres::audit_store::{AuditStoreError, PgAuditStore};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayoutInstructionStoreError {
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(#[from] WalletAddressError),
    #[error("Invalid token symbol: {0}")]
    InvalidTokenSymbol(#[from] TreasuryAccountError),
    #[error("Invalid treasury chain: {0}")]
    InvalidChain(#[from] ParseTreasuryChainError),
    #[error("Invalid treasury custody provider: {0}")]
    InvalidCustodyProvider(#[from] ParseTreasuryCustodyProviderError),
    #[error("Invalid treasury control mode: {0}")]
    InvalidControlMode(#[from] ParseTreasuryControlModeError),
    #[error("Invalid payout instruction: {0}")]
    InvalidPayoutInstruction(#[from] PayoutInstructionError),
    #[error("Invalid compensation amount: {0}")]
    InvalidCompensationAmount(#[from] CompensationProfileError),
    #[error("Invalid amount units")]
    InvalidAmountUnits,
    #[error("Invalid chain id: {0}")]
    InvalidChainId(i64),
    #[error("Invalid token decimals: {0}")]
    InvalidTokenDecimals(i16),
    #[error("audit event count must match instruction count")]
    AuditEventCountMismatch,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditStoreError),
}

#[derive(Debug, Clone)]
pub struct PgPayoutInstructionStore {
    pool: PgPool,
}

impl PgPayoutInstructionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PayoutInstructionRow {
    id: String,
    tenant_id: String,
    payrun_id: String,
    payrun_item_id: String,
    employee_id: String,
    treasury_account_id: String,
    idempotency_key: String,
    destination_wallet_address: String,
    source_wallet_address: String,
    chain: String,
    chain_id: i64,
    token_symbol: String,
    token_address: String,
    token_decimals: i16,
    amount_units: String,
    custody_provider: String,
    control_mode: String,
    provider_wallet_id: Option<String>,
    provider_owner_id: Option<String>,
    secret_reference: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<&PayoutInstruction> for PayoutInstructionRow {
    fn from(instruction: &PayoutInstruction) -> Self {
        Self {
            id: instruction.id().to_string(),
            tenant_id: instruction.tenant_id().to_string(),
            payrun_id: instruction.payrun_id().to_string(),
            payrun_item_id: instruction.payrun_item_id().to_string(),
            employee_id: instruction.employee_id().to_string(),
            treasury_account_id: instruction.treasury_account_id().to_string(),
            idempotency_key: instruction.idempotency_key().as_str().to_string(),
            destination_wallet_address: instruction
                .destination_wallet_address()
                .as_str()
                .to_string(),
            source_wallet_address: instruction.source_wallet_address().as_str().to_string(),
            chain: instruction.chain().to_string(),
            chain_id: instruction.chain_id() as i64,
            token_symbol: instruction.token_symbol().to_string(),
            token_address: instruction.token_address().as_str().to_string(),
            token_decimals: i16::from(instruction.token_decimals()),
            amount_units: instruction.amount().amount_units().to_string(),
            custody_provider: instruction.custody_provider().to_string(),
            control_mode: instruction.control_mode().to_string(),
            provider_wallet_id: instruction.provider_wallet_id().map(str::to_string),
            provider_owner_id: instruction.provider_owner_id().map(str::to_string),
            secret_reference: instruction.secret_reference().map(str::to_string),
            created_at: instruction.created_at(),
        }
    }
}

impl TryFrom<PayoutInstructionRow> for PayoutInstruction {
    type Error = PayoutInstructionStoreError;

    fn try_from(row: PayoutInstructionRow) -> Result<Self, Self::Error> {
        let amount_units = row
            .amount_units
            .parse::<u128>()
            .map_err(|_| PayoutInstructionStoreError::InvalidAmountUnits)?;
        let token_symbol = TokenSymbol::parse(row.token_symbol)?;
        let amount = CompensationAmount::new(amount_units, token_symbol.clone())?;
        let chain_id = u64::try_from(row.chain_id)
            .map_err(|_| PayoutInstructionStoreError::InvalidChainId(row.chain_id))?;
        let token_decimals = u8::try_from(row.token_decimals)
            .map_err(|_| PayoutInstructionStoreError::InvalidTokenDecimals(row.token_decimals))?;

        PayoutInstruction::restore(
            StandardID::<IDPayoutInstruction>::from_str(&row.id)?,
            PayoutInstructionDraft {
                tenant_id: StandardID::<IDTenant>::from_str(&row.tenant_id)?,
                payrun_id: StandardID::<IDPayrun>::from_str(&row.payrun_id)?,
                payrun_item_id: StandardID::<IDPayrunItem>::from_str(&row.payrun_item_id)?,
                employee_id: StandardID::<IDEmployee>::from_str(&row.employee_id)?,
                treasury_account_id: StandardID::<IDTreasuryAccount>::from_str(
                    &row.treasury_account_id,
                )?,
                idempotency_key: PayoutInstructionIdempotencyKey::parse(row.idempotency_key)?,
                destination_wallet_address: WalletAddress::parse(row.destination_wallet_address)?,
                source_wallet_address: WalletAddress::parse(row.source_wallet_address)?,
                chain: TreasuryChain::from_str(&row.chain)?,
                chain_id,
                token_symbol,
                token_address: WalletAddress::parse(row.token_address)?,
                token_decimals,
                amount,
                custody_provider: TreasuryCustodyProvider::from_str(&row.custody_provider)?,
                control_mode: TreasuryControlMode::from_str(&row.control_mode)?,
                provider_wallet_id: row.provider_wallet_id,
                provider_owner_id: row.provider_owner_id,
                secret_reference: row.secret_reference,
            },
            row.created_at,
        )
        .map_err(Into::into)
    }
}

impl PayoutInstructionStore for PgPayoutInstructionStore {
    async fn create_many_idempotent(
        &self,
        instructions: &[PayoutInstruction],
        audit_events: &[AuditEvent],
    ) -> Result<Vec<PayoutInstruction>, PayoutInstructionStoreError> {
        if instructions.len() != audit_events.len() {
            return Err(PayoutInstructionStoreError::AuditEventCountMismatch);
        }

        let Some(first) = instructions.first() else {
            return Ok(Vec::new());
        };

        let tenant_id = first.tenant_id();
        let payrun_id = first.payrun_id();
        let mut tx = self.pool.begin().await?;

        for (instruction, audit_event) in instructions.iter().zip(audit_events) {
            let row = PayoutInstructionRow::from(instruction);
            let inserted_id = sqlx::query_scalar::<_, String>(
                r#"
                INSERT INTO payout_instructions
                    (id, tenant_id, payrun_id, payrun_item_id, employee_id, treasury_account_id,
                     idempotency_key, destination_wallet_address, source_wallet_address,
                     chain, chain_id, token_symbol, token_address, token_decimals, amount_units,
                     custody_provider, control_mode, provider_wallet_id, provider_owner_id,
                     secret_reference, created_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                     $15::numeric, $16, $17, $18, $19, $20, $21)
                ON CONFLICT (idempotency_key) DO NOTHING
                RETURNING id
                "#,
            )
            .bind(&row.id)
            .bind(&row.tenant_id)
            .bind(&row.payrun_id)
            .bind(&row.payrun_item_id)
            .bind(&row.employee_id)
            .bind(&row.treasury_account_id)
            .bind(&row.idempotency_key)
            .bind(&row.destination_wallet_address)
            .bind(&row.source_wallet_address)
            .bind(&row.chain)
            .bind(row.chain_id)
            .bind(&row.token_symbol)
            .bind(&row.token_address)
            .bind(row.token_decimals)
            .bind(&row.amount_units)
            .bind(&row.custody_provider)
            .bind(&row.control_mode)
            .bind(row.provider_wallet_id.as_deref())
            .bind(row.provider_owner_id.as_deref())
            .bind(row.secret_reference.as_deref())
            .bind(row.created_at)
            .fetch_optional(&mut *tx)
            .await?;

            if inserted_id.is_some() {
                PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
            }
        }

        tx.commit().await?;
        self.list_for_payrun(tenant_id, payrun_id).await
    }

    async fn list_for_payrun(
        &self,
        tenant_id: &StandardID<IDTenant>,
        payrun_id: &StandardID<IDPayrun>,
    ) -> Result<Vec<PayoutInstruction>, PayoutInstructionStoreError> {
        let rows = sqlx::query_as::<sqlx::Postgres, PayoutInstructionRow>(
            r#"
            SELECT
                id,
                tenant_id,
                payrun_id,
                payrun_item_id,
                employee_id,
                treasury_account_id,
                idempotency_key,
                destination_wallet_address,
                source_wallet_address,
                chain,
                chain_id,
                token_symbol,
                token_address,
                token_decimals,
                amount_units::text AS amount_units,
                custody_provider,
                control_mode,
                provider_wallet_id,
                provider_owner_id,
                secret_reference,
                created_at
            FROM payout_instructions
            WHERE tenant_id = $1 AND payrun_id = $2
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(payrun_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(PayoutInstruction::try_from).collect()
    }

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayoutInstruction>,
    ) -> Result<Option<PayoutInstruction>, PayoutInstructionStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, PayoutInstructionRow>(
            r#"
            SELECT
                id,
                tenant_id,
                payrun_id,
                payrun_item_id,
                employee_id,
                treasury_account_id,
                idempotency_key,
                destination_wallet_address,
                source_wallet_address,
                chain,
                chain_id,
                token_symbol,
                token_address,
                token_decimals,
                amount_units::text AS amount_units,
                custody_provider,
                control_mode,
                provider_wallet_id,
                provider_owner_id,
                secret_reference,
                created_at
            FROM payout_instructions
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(PayoutInstruction::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
    use crate::domain::employee::Employee;
    use crate::domain::payrun::{CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{TreasuryAccount, TreasuryAccountDraft};
    use crate::domain::user::IDUser;
    use crate::services::datastore::PayrunStore;
    use crate::services::datastore::postgres::payrun_store::PgPayrunStore;
    use serde_json::json;

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn amount(token_symbol: &str) -> CompensationAmount {
        CompensationAmount::new(1_000_000, TokenSymbol::parse(token_symbol).unwrap()).unwrap()
    }

    fn payrun() -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    amount("USDC"),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap()
    }

    fn employee(id: StandardID<IDEmployee>) -> Employee {
        Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(id)
            .with_wallet_address(Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")))
    }

    fn treasury_account(tenant_id: StandardID<IDTenant>) -> TreasuryAccount {
        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id,
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
        .unwrap()
    }

    fn instruction() -> PayoutInstruction {
        let (payrun, instruction) = payrun_with_instruction();
        drop(payrun);
        instruction
    }

    fn payrun_with_instruction() -> (Payrun, PayoutInstruction) {
        let payrun = payrun();
        let item = &payrun.items()[0];
        let instruction = PayoutInstruction::new(
            &payrun,
            item,
            &employee(*item.employee_id()),
            &treasury_account(*payrun.tenant_id()),
        )
        .unwrap();

        (payrun, instruction)
    }

    fn audit_event(instruction: &PayoutInstruction) -> AuditEvent {
        AuditEvent::new(
            *instruction.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::PayoutInstruction,
            instruction.id().to_string(),
            AuditEventType::PayoutInstructionCreated,
            json!({ "payout_instruction": instruction }),
        )
    }

    fn payrun_audit_event(payrun: &Payrun) -> AuditEvent {
        AuditEvent::new(
            *payrun.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::Payrun,
            payrun.id().to_string(),
            AuditEventType::PayrunCreated,
            json!({ "payrun": payrun }),
        )
    }

    async fn persist_payrun(pool: PgPool, payrun: &Payrun) {
        PgPayrunStore::new(pool)
            .create(payrun, &payrun_audit_event(payrun))
            .await
            .unwrap();
    }

    #[test]
    fn payout_instruction_to_row_roundtrip() {
        let instruction = instruction();
        let row = PayoutInstructionRow::from(&instruction);

        assert_eq!(row.idempotency_key, instruction.idempotency_key().as_str());
        assert_eq!(row.amount_units, "1000000");
        assert_eq!(row.chain, "tempo-testnet");
        assert_eq!(row.control_mode, "server_controlled");

        let back = PayoutInstruction::try_from(row).unwrap();

        assert_eq!(back.id(), instruction.id());
        assert_eq!(back.payrun_item_id(), instruction.payrun_item_id());
        assert_eq!(back.token_symbol().as_str(), "USDC");
    }

    #[sqlx::test]
    async fn creates_instruction_and_audit_event(pool: PgPool) {
        let store = PgPayoutInstructionStore::new(pool.clone());
        let (payrun, instruction) = payrun_with_instruction();
        persist_payrun(pool.clone(), &payrun).await;
        let event = audit_event(&instruction);

        let created = store
            .create_many_idempotent(std::slice::from_ref(&instruction), &[event])
            .await
            .unwrap();

        assert_eq!(created.len(), 1);
        assert_eq!(created[0].idempotency_key(), instruction.idempotency_key());

        let (audit_count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(instruction.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[sqlx::test]
    async fn create_many_is_idempotent(pool: PgPool) {
        let store = PgPayoutInstructionStore::new(pool.clone());
        let (payrun, instruction) = payrun_with_instruction();
        persist_payrun(pool.clone(), &payrun).await;
        let event = audit_event(&instruction);

        store
            .create_many_idempotent(
                std::slice::from_ref(&instruction),
                std::slice::from_ref(&event),
            )
            .await
            .unwrap();
        let created_again = store
            .create_many_idempotent(
                std::slice::from_ref(&instruction),
                std::slice::from_ref(&event),
            )
            .await
            .unwrap();

        assert_eq!(created_again.len(), 1);

        let (instruction_count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM payout_instructions")
                .fetch_one(&pool)
                .await
                .unwrap();
        let (audit_count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(instruction.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(instruction_count, 1);
        assert_eq!(audit_count, 1);
    }

    #[sqlx::test]
    async fn lists_instructions_for_payrun(pool: PgPool) {
        let store = PgPayoutInstructionStore::new(pool.clone());
        let (payrun, instruction) = payrun_with_instruction();
        persist_payrun(pool, &payrun).await;

        store
            .create_many_idempotent(
                std::slice::from_ref(&instruction),
                &[audit_event(&instruction)],
            )
            .await
            .unwrap();

        let found = store
            .list_for_payrun(instruction.tenant_id(), instruction.payrun_id())
            .await
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id(), instruction.id());
    }
}
