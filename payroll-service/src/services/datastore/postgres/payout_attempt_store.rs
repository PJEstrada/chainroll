use crate::domain::audit::AuditEvent;
use crate::domain::ids::{IdError, StandardID};
use crate::domain::payout_attempt::{
    IDPayoutAttempt, ParsePayoutAttemptStatusError, ParsePayoutProviderError,
    ParsePayoutSignerProviderError, PayoutAttempt, PayoutAttemptDraft, PayoutAttemptError,
    PayoutAttemptStatus, PayoutProvider, PayoutSignerProvider,
};
use crate::domain::payout_instruction::IDPayoutInstruction;
use crate::domain::payrun::IDPayrun;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::PayoutAttemptStore;
use crate::services::datastore::postgres::audit_store::{AuditStoreError, PgAuditStore};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayoutAttemptStoreError {
    #[error("payout attempt not found")]
    PayoutAttemptNotFound,
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid payout attempt status: {0}")]
    InvalidStatus(#[from] ParsePayoutAttemptStatusError),
    #[error("Invalid payout provider: {0}")]
    InvalidProvider(#[from] ParsePayoutProviderError),
    #[error("Invalid payout signer provider: {0}")]
    InvalidSignerProvider(#[from] ParsePayoutSignerProviderError),
    #[error("Invalid payout attempt: {0}")]
    InvalidPayoutAttempt(#[from] PayoutAttemptError),
    #[error("Invalid attempt number: {0}")]
    InvalidAttemptNumber(i32),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditStoreError),
}

#[derive(Debug, Clone)]
pub struct PgPayoutAttemptStore {
    pool: PgPool,
}

impl PgPayoutAttemptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PayoutAttemptRow {
    id: String,
    tenant_id: String,
    payrun_id: String,
    payout_instruction_id: String,
    attempt_number: i32,
    status: String,
    provider: String,
    signer_provider: String,
    provider_reference: Option<String>,
    transaction_hash: Option<String>,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<&PayoutAttempt> for PayoutAttemptRow {
    fn from(attempt: &PayoutAttempt) -> Self {
        Self {
            id: attempt.id().to_string(),
            tenant_id: attempt.tenant_id().to_string(),
            payrun_id: attempt.payrun_id().to_string(),
            payout_instruction_id: attempt.payout_instruction_id().to_string(),
            attempt_number: attempt.attempt_number() as i32,
            status: attempt.status().to_string(),
            provider: attempt.provider().to_string(),
            signer_provider: attempt.signer_provider().to_string(),
            provider_reference: attempt.provider_reference().map(str::to_string),
            transaction_hash: attempt.transaction_hash().map(str::to_string),
            error_message: attempt.error_message().map(str::to_string),
            started_at: attempt.started_at(),
            completed_at: attempt.completed_at(),
        }
    }
}

impl TryFrom<PayoutAttemptRow> for PayoutAttempt {
    type Error = PayoutAttemptStoreError;

    fn try_from(row: PayoutAttemptRow) -> Result<Self, Self::Error> {
        let attempt_number = u32::try_from(row.attempt_number)
            .map_err(|_| PayoutAttemptStoreError::InvalidAttemptNumber(row.attempt_number))?;

        PayoutAttempt::restore(
            StandardID::<IDPayoutAttempt>::from_str(&row.id)?,
            PayoutAttemptDraft {
                tenant_id: StandardID::<IDTenant>::from_str(&row.tenant_id)?,
                payrun_id: StandardID::<IDPayrun>::from_str(&row.payrun_id)?,
                payout_instruction_id: StandardID::<IDPayoutInstruction>::from_str(
                    &row.payout_instruction_id,
                )?,
                attempt_number,
                status: PayoutAttemptStatus::from_str(&row.status)?,
                provider: PayoutProvider::from_str(&row.provider)?,
                signer_provider: PayoutSignerProvider::from_str(&row.signer_provider)?,
                provider_reference: row.provider_reference,
                transaction_hash: row.transaction_hash,
                error_message: row.error_message,
                started_at: row.started_at,
                completed_at: row.completed_at,
            },
        )
        .map_err(Into::into)
    }
}

impl PayoutAttemptStore for PgPayoutAttemptStore {
    async fn create_started(
        &self,
        attempt: &PayoutAttempt,
        audit_event: &AuditEvent,
    ) -> Result<PayoutAttempt, PayoutAttemptStoreError> {
        let row = PayoutAttemptRow::from(attempt);
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO payout_attempts
                (id, tenant_id, payrun_id, payout_instruction_id, attempt_number, status,
                 provider, signer_provider, provider_reference, transaction_hash, error_message,
                 started_at, completed_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.payrun_id)
        .bind(&row.payout_instruction_id)
        .bind(row.attempt_number)
        .bind(&row.status)
        .bind(&row.provider)
        .bind(&row.signer_provider)
        .bind(row.provider_reference.as_deref())
        .bind(row.transaction_hash.as_deref())
        .bind(row.error_message.as_deref())
        .bind(row.started_at)
        .bind(row.completed_at)
        .execute(&mut *tx)
        .await?;

        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        Ok(attempt.clone())
    }

    async fn update_final(
        &self,
        attempt: &PayoutAttempt,
        audit_event: &AuditEvent,
    ) -> Result<PayoutAttempt, PayoutAttemptStoreError> {
        let row = PayoutAttemptRow::from(attempt);
        let mut tx = self.pool.begin().await?;

        let updated = sqlx::query_as::<sqlx::Postgres, PayoutAttemptRow>(
            r#"
            UPDATE payout_attempts
            SET
                status = $3,
                provider_reference = $4,
                transaction_hash = $5,
                error_message = $6,
                completed_at = $7
            WHERE tenant_id = $1 AND id = $2 AND status = 'started'
            RETURNING
                id,
                tenant_id,
                payrun_id,
                payout_instruction_id,
                attempt_number,
                status,
                provider,
                signer_provider,
                provider_reference,
                transaction_hash,
                error_message,
                started_at,
                completed_at
            "#,
        )
        .bind(&row.tenant_id)
        .bind(&row.id)
        .bind(&row.status)
        .bind(row.provider_reference.as_deref())
        .bind(row.transaction_hash.as_deref())
        .bind(row.error_message.as_deref())
        .bind(row.completed_at)
        .fetch_optional(&mut *tx)
        .await?;

        let updated = updated.ok_or(PayoutAttemptStoreError::PayoutAttemptNotFound)?;
        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        PayoutAttempt::try_from(updated)
    }

    async fn list_for_payrun(
        &self,
        tenant_id: &StandardID<IDTenant>,
        payrun_id: &StandardID<IDPayrun>,
    ) -> Result<Vec<PayoutAttempt>, PayoutAttemptStoreError> {
        let rows = sqlx::query_as::<sqlx::Postgres, PayoutAttemptRow>(
            r#"
            SELECT
                id,
                tenant_id,
                payrun_id,
                payout_instruction_id,
                attempt_number,
                status,
                provider,
                signer_provider,
                provider_reference,
                transaction_hash,
                error_message,
                started_at,
                completed_at
            FROM payout_attempts
            WHERE tenant_id = $1 AND payrun_id = $2
            ORDER BY started_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(payrun_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(PayoutAttempt::try_from).collect()
    }

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayoutAttempt>,
    ) -> Result<Option<PayoutAttempt>, PayoutAttemptStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, PayoutAttemptRow>(
            r#"
            SELECT
                id,
                tenant_id,
                payrun_id,
                payout_instruction_id,
                attempt_number,
                status,
                provider,
                signer_provider,
                provider_reference,
                transaction_hash,
                error_message,
                started_at,
                completed_at
            FROM payout_attempts
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(PayoutAttempt::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::{Employee, IDEmployee};
    use crate::domain::payout_instruction::PayoutInstruction;
    use crate::domain::payrun::{CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::user::IDUser;
    use crate::domain::wallets::WalletAddress;
    use crate::services::datastore::postgres::payout_instruction_store::PgPayoutInstructionStore;
    use crate::services::datastore::postgres::payrun_store::PgPayrunStore;
    use crate::services::datastore::{PayoutInstructionStore, PayrunStore};
    use serde_json::json;

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn payrun_with_instruction() -> (Payrun, PayoutInstruction) {
        let payrun = Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::<IDEmployee>::new(),
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
            custody_provider: TreasuryCustodyProvider::Privy,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: Some("privy-wallet-id".to_string()),
            provider_owner_id: None,
            secret_reference: None,
            is_default: true,
        })
        .unwrap();
        let instruction = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap();

        (payrun, instruction)
    }

    fn audit_event(attempt: &PayoutAttempt, event_type: AuditEventType) -> AuditEvent {
        AuditEvent::new(
            *attempt.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::PayoutAttempt,
            attempt.id().to_string(),
            event_type,
            json!({ "payout_attempt": attempt }),
        )
    }

    async fn persist_instruction(pool: PgPool, payrun: &Payrun, instruction: &PayoutInstruction) {
        let payrun_event = AuditEvent::new(
            *payrun.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::Payrun,
            payrun.id().to_string(),
            AuditEventType::PayrunCreated,
            json!({ "payrun": payrun }),
        );
        PgPayrunStore::new(pool.clone())
            .create(payrun, &payrun_event)
            .await
            .unwrap();

        let instruction_event = AuditEvent::new(
            *instruction.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::PayoutInstruction,
            instruction.id().to_string(),
            AuditEventType::PayoutInstructionCreated,
            json!({ "payout_instruction": instruction }),
        );
        PgPayoutInstructionStore::new(pool)
            .create_many_idempotent(std::slice::from_ref(instruction), &[instruction_event])
            .await
            .unwrap();
    }

    #[test]
    fn payout_attempt_row_roundtrip() {
        let attempt =
            PayoutAttempt::started(StandardID::new(), StandardID::new(), StandardID::new(), 1)
                .unwrap();
        let row = PayoutAttemptRow::from(&attempt);

        assert_eq!(row.status, "started");
        assert_eq!(row.provider, "tempo");
        assert_eq!(row.signer_provider, "privy");

        let back = PayoutAttempt::try_from(row).unwrap();

        assert_eq!(back.id(), attempt.id());
        assert_eq!(back.status(), PayoutAttemptStatus::Started);
    }

    #[sqlx::test]
    async fn creates_started_attempt_and_audit_event(pool: PgPool) {
        let store = PgPayoutAttemptStore::new(pool.clone());
        let (payrun, instruction) = payrun_with_instruction();
        persist_instruction(pool.clone(), &payrun, &instruction).await;
        let attempt = PayoutAttempt::started(
            *instruction.tenant_id(),
            *instruction.payrun_id(),
            *instruction.id(),
            1,
        )
        .unwrap();

        let created = store
            .create_started(
                &attempt,
                &audit_event(&attempt, AuditEventType::PayoutAttemptStarted),
            )
            .await
            .unwrap();

        assert_eq!(created.status(), PayoutAttemptStatus::Started);

        let (audit_count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(attempt.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[sqlx::test]
    async fn updates_started_attempt_to_final_status(pool: PgPool) {
        let store = PgPayoutAttemptStore::new(pool.clone());
        let (payrun, instruction) = payrun_with_instruction();
        persist_instruction(pool.clone(), &payrun, &instruction).await;
        let attempt = PayoutAttempt::started(
            *instruction.tenant_id(),
            *instruction.payrun_id(),
            *instruction.id(),
            1,
        )
        .unwrap();
        store
            .create_started(
                &attempt,
                &audit_event(&attempt, AuditEventType::PayoutAttemptStarted),
            )
            .await
            .unwrap();
        let tx_hash =
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        let submitted = attempt
            .mark_submitted(tx_hash.clone(), Some(tx_hash.clone()))
            .unwrap();

        let updated = store
            .update_final(
                &submitted,
                &audit_event(&submitted, AuditEventType::PayoutAttemptSubmitted),
            )
            .await
            .unwrap();

        assert_eq!(updated.status(), PayoutAttemptStatus::Submitted);
        assert_eq!(updated.transaction_hash(), Some(tx_hash.as_str()));

        let attempts = store
            .list_for_payrun(instruction.tenant_id(), instruction.payrun_id())
            .await
            .unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status(), PayoutAttemptStatus::Submitted);
    }
}
