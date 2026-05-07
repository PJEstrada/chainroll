use crate::domain::audit::AuditEvent;
use crate::domain::compensation::CompensationAmount;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::{IdError, StandardID};
use crate::domain::payrun::{
    IDPayrun, IDPayrunItem, ParsePayrunItemStatusError, ParsePayrunStatusError, Payrun,
    PayrunError, PayrunItem, PayrunItemStatus, PayrunPreviewBlocker, PayrunStatus,
};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{TokenSymbol, TreasuryAccountError};
use crate::services::datastore::PayrunStore;
use crate::services::datastore::postgres::audit_store::{AuditStoreError, PgAuditStore};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayrunStoreError {
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid payrun status: {0}")]
    InvalidPayrunStatus(#[from] ParsePayrunStatusError),
    #[error("Invalid payrun item status: {0}")]
    InvalidPayrunItemStatus(#[from] ParsePayrunItemStatusError),
    #[error("Invalid token symbol: {0}")]
    InvalidTokenSymbol(#[from] TreasuryAccountError),
    #[error("Invalid amount units")]
    InvalidAmountUnits,
    #[error("Invalid payrun: {0}")]
    InvalidPayrun(#[from] PayrunError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditStoreError),
}

#[derive(Debug, Clone)]
pub struct PgPayrunStore {
    pool: PgPool,
}

impl PgPayrunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PayrunRow {
    id: String,
    tenant_id: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct PayrunItemRow {
    id: String,
    payrun_id: String,
    tenant_id: String,
    employee_id: String,
    status: String,
    amount_units: Option<String>,
    token_symbol: Option<String>,
    #[sqlx(json)]
    blockers: Vec<PayrunPreviewBlocker>,
    created_at: DateTime<Utc>,
}

impl From<&Payrun> for PayrunRow {
    fn from(payrun: &Payrun) -> Self {
        Self {
            id: payrun.id().to_string(),
            tenant_id: payrun.tenant_id().to_string(),
            status: payrun.status().to_string(),
            created_at: payrun.created_at(),
            updated_at: payrun.updated_at(),
        }
    }
}

impl PayrunItemRow {
    fn from_item(payrun: &Payrun, item: &PayrunItem) -> Self {
        Self {
            id: item.id().to_string(),
            payrun_id: payrun.id().to_string(),
            tenant_id: payrun.tenant_id().to_string(),
            employee_id: item.employee_id().to_string(),
            status: item.status().to_string(),
            amount_units: item
                .amount()
                .map(|amount| amount.amount_units().to_string()),
            token_symbol: item
                .amount()
                .map(|amount| amount.token_symbol().to_string()),
            blockers: item.blockers().to_vec(),
            created_at: payrun.created_at(),
        }
    }

    fn into_item(self) -> Result<PayrunItem, PayrunStoreError> {
        let amount = match (self.amount_units, self.token_symbol) {
            (Some(amount_units), Some(token_symbol)) => {
                let amount_units = amount_units
                    .parse::<u128>()
                    .map_err(|_| PayrunStoreError::InvalidAmountUnits)?;
                Some(
                    CompensationAmount::new(amount_units, TokenSymbol::parse(token_symbol)?)
                        .map_err(|_| PayrunStoreError::InvalidAmountUnits)?,
                )
            }
            (None, None) => None,
            _ => return Err(PayrunStoreError::InvalidAmountUnits),
        };

        PayrunItem::restore(
            StandardID::<IDPayrunItem>::from_str(&self.id)?,
            StandardID::<IDEmployee>::from_str(&self.employee_id)?,
            PayrunItemStatus::from_str(&self.status)?,
            amount,
            self.blockers,
        )
        .map_err(Into::into)
    }
}

impl PayrunStore for PgPayrunStore {
    async fn create(
        &self,
        payrun: &Payrun,
        audit_event: &AuditEvent,
    ) -> Result<Payrun, PayrunStoreError> {
        let row = PayrunRow::from(payrun);
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO payruns (id, tenant_id, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.status)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut *tx)
        .await?;

        for item in payrun.items() {
            let item_row = PayrunItemRow::from_item(payrun, item);
            sqlx::query(
                r#"
                INSERT INTO payrun_items
                    (id, payrun_id, tenant_id, employee_id, status, amount_units, token_symbol, blockers, created_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9)
                "#,
            )
            .bind(&item_row.id)
            .bind(&item_row.payrun_id)
            .bind(&item_row.tenant_id)
            .bind(&item_row.employee_id)
            .bind(&item_row.status)
            .bind(item_row.amount_units.as_deref())
            .bind(item_row.token_symbol.as_deref())
            .bind(sqlx::types::Json(&item_row.blockers))
            .bind(item_row.created_at)
            .execute(&mut *tx)
            .await?;
        }

        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        Ok(payrun.clone())
    }

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayrun>,
    ) -> Result<Option<Payrun>, PayrunStoreError> {
        let Some(row) = sqlx::query_as::<sqlx::Postgres, PayrunRow>(
            r#"
            SELECT id, tenant_id, status, created_at, updated_at
            FROM payruns
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let item_rows = sqlx::query_as::<sqlx::Postgres, PayrunItemRow>(
            r#"
            SELECT
                id,
                payrun_id,
                tenant_id,
                employee_id,
                status,
                amount_units::text AS amount_units,
                token_symbol,
                blockers,
                created_at
            FROM payrun_items
            WHERE tenant_id = $1 AND payrun_id = $2
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let items = item_rows
            .into_iter()
            .map(PayrunItemRow::into_item)
            .collect::<Result<Vec<_>, _>>()?;

        Payrun::restore(
            StandardID::<IDPayrun>::from_str(&row.id)?,
            StandardID::<IDTenant>::from_str(&row.tenant_id)?,
            PayrunStatus::from_str(&row.status)?,
            items,
            row.created_at,
            row.updated_at,
        )
        .map(Some)
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
    use crate::domain::payrun::{
        CreatePayrunOptions, PayrunPreview, PayrunPreviewBlocker, PayrunPreviewItem,
    };
    use crate::domain::user::IDUser;
    use serde_json::json;

    fn amount(amount_units: u128, token_symbol: &str) -> CompensationAmount {
        CompensationAmount::new(amount_units, TokenSymbol::parse(token_symbol).unwrap()).unwrap()
    }

    fn payrun() -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![
                    PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                    PayrunPreviewItem::blocked(
                        StandardID::new(),
                        Some(amount(2_000_000, "USDC")),
                        vec![PayrunPreviewBlocker::MissingWallet],
                    )
                    .unwrap(),
                ],
            ),
            CreatePayrunOptions::exclude_unpayable(),
        )
        .unwrap()
    }

    fn audit_event(payrun: &Payrun) -> AuditEvent {
        AuditEvent::new(
            *payrun.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::Payrun,
            payrun.id().to_string(),
            AuditEventType::PayrunCreated,
            json!({ "payrun": payrun }),
        )
    }

    #[sqlx::test]
    async fn creates_payrun_items_and_audit_event(pool: PgPool) {
        let store = PgPayrunStore::new(pool.clone());
        let payrun = payrun();

        let created = store.create(&payrun, &audit_event(&payrun)).await.unwrap();

        assert_eq!(created.id(), payrun.id());
        assert_eq!(created.items().len(), 2);

        let (audit_count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(payrun.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[sqlx::test]
    async fn gets_payrun_with_items(pool: PgPool) {
        let store = PgPayrunStore::new(pool);
        let payrun = payrun();

        store.create(&payrun, &audit_event(&payrun)).await.unwrap();

        let found = store
            .get(payrun.tenant_id(), payrun.id())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.id(), payrun.id());
        assert_eq!(found.items().len(), 2);
        assert_eq!(found.items()[0].status(), PayrunItemStatus::Payable);
        assert_eq!(found.items()[1].status(), PayrunItemStatus::Excluded);
        assert_eq!(
            found.totals().unwrap().total_amounts()[0].amount_units(),
            1_000_000
        );
    }
}
