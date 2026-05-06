use crate::domain::audit::AuditEvent;
use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus, ParseStatusError};
use crate::domain::ids::{IdError, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{
    IDTreasuryAccount, ParseTreasuryChainError, ParseTreasuryControlModeError,
    ParseTreasuryCustodyProviderError, TokenSymbol, TreasuryAccount, TreasuryAccountDraft,
    TreasuryAccountError, TreasuryAccountQuery, TreasuryChain, TreasuryControlMode,
    TreasuryCustodyProvider,
};
use crate::domain::wallets::{WalletAddress, WalletAddressError};
use crate::services::datastore::TreasuryStore;
use crate::services::datastore::postgres::audit_store::{AuditStoreError, PgAuditStore};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(sqlx::FromRow)]
struct TreasuryAccountRow {
    id: String,
    tenant_id: String,
    name: String,
    chain: String,
    chain_id: i64,
    token_symbol: String,
    token_address: String,
    token_decimals: i16,
    sender_address: String,
    custody_provider: String,
    control_mode: String,
    provider_wallet_id: Option<String>,
    provider_owner_id: Option<String>,
    secret_reference: Option<String>,
    status: String,
    is_default: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<TreasuryAccountRow> for TreasuryAccount {
    type Error = TreasuryStoreError;

    fn try_from(row: TreasuryAccountRow) -> Result<Self, Self::Error> {
        let id = StandardID::<IDTreasuryAccount>::from_str(row.id.as_str())?;
        let tenant_id = StandardID::<IDTenant>::from_str(row.tenant_id.as_str())?;
        let chain = TreasuryChain::from_str(row.chain.as_str())?;
        let chain_id =
            u64::try_from(row.chain_id).map_err(|_| TreasuryStoreError::InvalidChainId {
                expected: chain.chain_id(),
                actual: row.chain_id,
            })?;

        if chain_id != chain.chain_id() {
            return Err(TreasuryStoreError::InvalidChainId {
                expected: chain.chain_id(),
                actual: row.chain_id,
            });
        }

        let token_decimals = u8::try_from(row.token_decimals).map_err(|_| {
            TreasuryStoreError::InvalidTokenDecimals {
                actual: row.token_decimals,
            }
        })?;

        TreasuryAccount::restore(
            id,
            LifecycleMeta {
                status: ObjectStatus::from_str(row.status.as_str())?,
                created: row.created_at,
                updated: row.updated_at,
            },
            TreasuryAccountDraft {
                tenant_id,
                name: row.name,
                chain,
                token_symbol: TokenSymbol::parse(row.token_symbol)?,
                token_address: WalletAddress::parse(row.token_address)?,
                token_decimals,
                sender_address: WalletAddress::parse(row.sender_address)?,
                custody_provider: TreasuryCustodyProvider::from_str(row.custody_provider.as_str())?,
                control_mode: TreasuryControlMode::from_str(row.control_mode.as_str())?,
                provider_wallet_id: row.provider_wallet_id,
                provider_owner_id: row.provider_owner_id,
                secret_reference: row.secret_reference,
                is_default: row.is_default,
            },
        )
        .map_err(Into::into)
    }
}

impl From<&TreasuryAccount> for TreasuryAccountRow {
    fn from(account: &TreasuryAccount) -> Self {
        Self {
            id: account.id().to_string(),
            tenant_id: account.tenant_id().to_string(),
            name: account.name().to_string(),
            chain: account.chain().to_string(),
            chain_id: account.chain_id() as i64,
            token_symbol: account.token_symbol().to_string(),
            token_address: account.token_address().as_str().to_string(),
            token_decimals: i16::from(account.token_decimals()),
            sender_address: account.sender_address().as_str().to_string(),
            custody_provider: account.custody_provider().to_string(),
            control_mode: account.control_mode().to_string(),
            provider_wallet_id: account.provider_wallet_id().map(str::to_string),
            provider_owner_id: account.provider_owner_id().map(str::to_string),
            secret_reference: account.secret_reference().map(str::to_string),
            status: account.status().to_string(),
            is_default: account.is_default(),
            created_at: account.metadata().created,
            updated_at: account.metadata().updated,
        }
    }
}

#[derive(Debug, Error)]
pub enum TreasuryStoreError {
    #[error("Treasury account not found")]
    TreasuryAccountNotFound,
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] ParseStatusError),
    #[error("Invalid treasury chain: {0}")]
    InvalidChain(#[from] ParseTreasuryChainError),
    #[error("Invalid treasury custody provider: {0}")]
    InvalidCustodyProvider(#[from] ParseTreasuryCustodyProviderError),
    #[error("Invalid treasury control mode: {0}")]
    InvalidControlMode(#[from] ParseTreasuryControlModeError),
    #[error("Invalid treasury account: {0}")]
    InvalidTreasuryAccount(#[from] TreasuryAccountError),
    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(#[from] WalletAddressError),
    #[error("Invalid chain id: expected {expected}, got {actual}")]
    InvalidChainId { expected: u64, actual: i64 },
    #[error("Invalid token decimals: {actual}")]
    InvalidTokenDecimals { actual: i16 },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditStoreError),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct PgTreasuryStore {
    pool: PgPool,
}

impl PgTreasuryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn query_limit(query: &TreasuryAccountQuery) -> Result<i64, TreasuryStoreError> {
    i64::try_from(query.base.limit.unwrap_or(100)).map_err(|err| anyhow::anyhow!(err).into())
}

fn query_offset(query: &TreasuryAccountQuery) -> Result<i64, TreasuryStoreError> {
    i64::try_from(query.base.offset.unwrap_or(0)).map_err(|err| anyhow::anyhow!(err).into())
}

async fn clear_existing_default(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account: &TreasuryAccount,
) -> Result<(), TreasuryStoreError> {
    if !account.is_default() || account.status() != ObjectStatus::Active {
        return Ok(());
    }

    sqlx::query(
        r#"
        UPDATE treasury_accounts
        SET is_default = FALSE,
            updated_at = now()
        WHERE tenant_id = $1
          AND chain = $2
          AND token_address = $3
          AND id <> $4
          AND status = 'active'
          AND is_default = TRUE
        "#,
    )
    .bind(account.tenant_id().to_string())
    .bind(account.chain().to_string())
    .bind(account.token_address().as_str())
    .bind(account.id().to_string())
    .execute(&mut **tx)
    .await?;

    Ok(())
}

impl TreasuryStore for PgTreasuryStore {
    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDTreasuryAccount>,
    ) -> Result<Option<TreasuryAccount>, TreasuryStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, TreasuryAccountRow>(
            "SELECT * FROM treasury_accounts WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(TreasuryAccount::try_from).transpose()
    }

    async fn list(
        &self,
        tenant_id: &StandardID<IDTenant>,
        query: &TreasuryAccountQuery,
    ) -> Result<Vec<TreasuryAccount>, TreasuryStoreError> {
        let status = query.status.map(|status| status.to_string());
        let chain = query.chain.map(|chain| chain.to_string());
        let limit = query_limit(query)?;
        let offset = query_offset(query)?;

        let rows = sqlx::query_as::<sqlx::Postgres, TreasuryAccountRow>(
            r#"
            SELECT * FROM treasury_accounts
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR chain = $3)
              AND ($4::boolean = FALSE OR is_default = TRUE)
            ORDER BY created_at ASC, id ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(status)
        .bind(chain)
        .bind(query.only_default)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TreasuryAccount::try_from).collect()
    }

    async fn list_default_active(
        &self,
        tenant_id: &StandardID<IDTenant>,
    ) -> Result<Vec<TreasuryAccount>, TreasuryStoreError> {
        let rows = sqlx::query_as::<sqlx::Postgres, TreasuryAccountRow>(
            r#"
            SELECT * FROM treasury_accounts
            WHERE tenant_id = $1
              AND status = 'active'
              AND is_default = TRUE
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TreasuryAccount::try_from).collect()
    }

    async fn create(
        &self,
        account: &TreasuryAccount,
        audit_event: &AuditEvent,
    ) -> Result<TreasuryAccount, TreasuryStoreError> {
        let row = TreasuryAccountRow::from(account);
        let mut tx = self.pool.begin().await?;

        clear_existing_default(&mut tx, account).await?;

        let created = sqlx::query_as::<sqlx::Postgres, TreasuryAccountRow>(
            r#"
            INSERT INTO treasury_accounts
                (id, tenant_id, name, chain, chain_id, token_symbol, token_address, token_decimals,
                 sender_address, custody_provider, control_mode, provider_wallet_id, provider_owner_id,
                 secret_reference, status, is_default, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.name)
        .bind(&row.chain)
        .bind(row.chain_id)
        .bind(&row.token_symbol)
        .bind(&row.token_address)
        .bind(row.token_decimals)
        .bind(&row.sender_address)
        .bind(&row.custody_provider)
        .bind(&row.control_mode)
        .bind(row.provider_wallet_id.as_deref())
        .bind(row.provider_owner_id.as_deref())
        .bind(row.secret_reference.as_deref())
        .bind(&row.status)
        .bind(row.is_default)
        .bind(row.created_at)
        .bind(row.updated_at)
        .fetch_one(&mut *tx)
        .await?;

        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        TreasuryAccount::try_from(created)
    }

    async fn update(
        &self,
        account: &TreasuryAccount,
        audit_event: &AuditEvent,
    ) -> Result<TreasuryAccount, TreasuryStoreError> {
        let row = TreasuryAccountRow::from(account);
        let mut tx = self.pool.begin().await?;

        clear_existing_default(&mut tx, account).await?;

        let updated = sqlx::query_as::<sqlx::Postgres, TreasuryAccountRow>(
            r#"
            UPDATE treasury_accounts
            SET name = $3,
                chain = $4,
                chain_id = $5,
                token_symbol = $6,
                token_address = $7,
                token_decimals = $8,
                sender_address = $9,
                custody_provider = $10,
                control_mode = $11,
                provider_wallet_id = $12,
                provider_owner_id = $13,
                secret_reference = $14,
                status = $15,
                is_default = $16,
                updated_at = now()
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.name)
        .bind(&row.chain)
        .bind(row.chain_id)
        .bind(&row.token_symbol)
        .bind(&row.token_address)
        .bind(row.token_decimals)
        .bind(&row.sender_address)
        .bind(&row.custody_provider)
        .bind(&row.control_mode)
        .bind(row.provider_wallet_id.as_deref())
        .bind(row.provider_owner_id.as_deref())
        .bind(row.secret_reference.as_deref())
        .bind(&row.status)
        .bind(row.is_default)
        .fetch_optional(&mut *tx)
        .await?;

        let updated = updated.ok_or(TreasuryStoreError::TreasuryAccountNotFound)?;
        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        TreasuryAccount::try_from(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditEntityType, AuditEventType};
    use crate::domain::user::IDUser;
    use serde_json::json;

    fn address(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn account(tenant_id: StandardID<IDTenant>, token_address: &str) -> TreasuryAccount {
        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id,
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("pathUSD").unwrap(),
            token_address: address(token_address),
            token_decimals: 18,
            sender_address: address("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        })
        .unwrap()
    }

    fn audit_event(account: &TreasuryAccount, event_type: AuditEventType) -> AuditEvent {
        AuditEvent::new(
            *account.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::TreasuryAccount,
            account.id().to_string(),
            event_type,
            json!({ "treasury_account": account }),
        )
    }

    #[test]
    fn treasury_account_to_row_roundtrip() {
        let tenant_id = StandardID::<IDTenant>::new();
        let account = account(tenant_id, "0x20c0000000000000000000000000000000000000");

        let row = TreasuryAccountRow::from(&account);

        assert_eq!(row.tenant_id, tenant_id.to_string());
        assert_eq!(row.chain, "tempo-testnet");
        assert_eq!(row.chain_id, 42431);
        assert_eq!(row.custody_provider, "local_key");
        assert_eq!(row.control_mode, "server_controlled");
        assert_eq!(row.status, "active");

        let back = TreasuryAccount::try_from(row).unwrap();
        assert_eq!(back.id(), account.id());
        assert_eq!(back.tenant_id(), account.tenant_id());
        assert_eq!(back.token_symbol().as_str(), "pathUSD");
        assert!(back.is_default());
    }

    #[sqlx::test]
    async fn creates_treasury_account_and_audit_event(pool: PgPool) {
        let store = PgTreasuryStore::new(pool.clone());
        let tenant_id = StandardID::<IDTenant>::new();
        let account = account(tenant_id, "0x20c0000000000000000000000000000000000000");
        let event = audit_event(&account, AuditEventType::TreasuryAccountCreated);

        let created = store.create(&account, &event).await.unwrap();

        assert_eq!(created.id(), account.id());
        assert!(created.is_default());

        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(account.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn only_one_default_active_account_per_tenant_chain_token(pool: PgPool) {
        let store = PgTreasuryStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let first = account(tenant_id, "0x20c0000000000000000000000000000000000000");
        let second = account(tenant_id, "0x20c0000000000000000000000000000000000000");

        store
            .create(
                &first,
                &audit_event(&first, AuditEventType::TreasuryAccountCreated),
            )
            .await
            .unwrap();
        store
            .create(
                &second,
                &audit_event(&second, AuditEventType::TreasuryAccountCreated),
            )
            .await
            .unwrap();

        let first = store.get(&tenant_id, first.id()).await.unwrap().unwrap();
        let second = store.get(&tenant_id, second.id()).await.unwrap().unwrap();

        assert!(!first.is_default());
        assert!(second.is_default());
    }

    #[sqlx::test]
    async fn lists_default_active_accounts_for_tenant(pool: PgPool) {
        let store = PgTreasuryStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let other_tenant_id = StandardID::<IDTenant>::new();
        let default_account = account(tenant_id, "0x20c0000000000000000000000000000000000000");
        let other_tenant = account(
            other_tenant_id,
            "0x30c0000000000000000000000000000000000000",
        );

        store
            .create(
                &default_account,
                &audit_event(&default_account, AuditEventType::TreasuryAccountCreated),
            )
            .await
            .unwrap();
        store
            .create(
                &other_tenant,
                &audit_event(&other_tenant, AuditEventType::TreasuryAccountCreated),
            )
            .await
            .unwrap();

        let accounts = store.list_default_active(&tenant_id).await.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id(), default_account.id());
        assert_eq!(accounts[0].tenant_id(), &tenant_id);
        assert!(accounts[0].is_default());
        assert_eq!(accounts[0].status(), ObjectStatus::Active);
    }

    #[sqlx::test]
    async fn updates_treasury_account(pool: PgPool) {
        let store = PgTreasuryStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let mut account = account(tenant_id, "0x20c0000000000000000000000000000000000000");

        account = store
            .create(
                &account,
                &audit_event(&account, AuditEventType::TreasuryAccountCreated),
            )
            .await
            .unwrap();
        account.clear_default();

        let updated = store
            .update(
                &account,
                &audit_event(&account, AuditEventType::TreasuryAccountUpdated),
            )
            .await
            .unwrap();

        assert!(!updated.is_default());
    }
}
