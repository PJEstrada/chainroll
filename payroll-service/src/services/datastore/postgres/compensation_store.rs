use crate::domain::audit::AuditEvent;
use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus, ParseStatusError};
use crate::domain::compensation::{
    CadenceUnit, CompensationAmount, CompensationCadence, CompensationProfile,
    CompensationProfileDraft, CompensationProfileError, IDCompensationProfile,
};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::{IdError, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TreasuryAccountError;
use crate::services::datastore::CompensationStore;
use crate::services::datastore::postgres::audit_store::{AuditStoreError, PgAuditStore};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct CompensationProfileRow {
    id: String,
    tenant_id: String,
    employee_id: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,

    amount_units: String,
    token_symbol: String,
    compensation_cadence: String,
    compensation_cadence_every: Option<i32>,
    compensation_cadence_unit: Option<String>,

    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PgCompensationStore {
    pool: PgPool,
}

impl PgCompensationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Error)]
pub enum CompensationStoreError {
    #[error("Compensation profile not found")]
    CompensationProfileNotFound,
    #[error("Compensation store already exists")]
    CompensationAlreadyExists,
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] ParseStatusError),
    #[error("Invalid token symbol: {0}")]
    InvalidTokenSymbol(#[from] TreasuryAccountError),
    #[error("Invalid compensation profile: {0}")]
    InvalidCompensationProfile(#[from] CompensationProfileError),
    #[error("Invalid amount units")]
    InvalidAmountUnits,
    #[error("Invalid cadence every")]
    InvalidCadenceEvery,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditStoreError),
}

impl TryFrom<CompensationProfileRow> for CompensationProfile {
    type Error = CompensationStoreError;

    fn try_from(row: CompensationProfileRow) -> Result<Self, Self::Error> {
        let id = StandardID::<IDCompensationProfile>::from_str(row.id.as_str())?;
        let tenant_id = StandardID::<IDTenant>::from_str(row.tenant_id.as_str())?;
        let employee_id = StandardID::<IDEmployee>::from_str(row.employee_id.as_str())?;
        let amount_units = row
            .amount_units
            .parse::<u128>()
            .map_err(|_| CompensationStoreError::InvalidAmountUnits)?;
        let token_symbol = row.token_symbol.parse()?;
        let compensation_cadence = parse_cadence_from_row(
            &row.compensation_cadence,
            row.compensation_cadence_every,
            row.compensation_cadence_unit.as_deref(),
        )?;
        let valid_from = row.valid_from;
        let valid_to = row.valid_to;
        CompensationProfile::restore(
            id,
            LifecycleMeta {
                status: ObjectStatus::from_str(row.status.as_str())?,
                created: row.created_at,
                updated: row.updated_at,
            },
            CompensationProfileDraft {
                amount: CompensationAmount {
                    amount_units,
                    token_symbol,
                },
                tenant_id,
                employee_id,
                cadence: compensation_cadence,
                valid_from,
                valid_to,
            },
        )
        .map_err(Into::into)
    }
}

impl From<&CompensationProfile> for CompensationProfileRow {
    fn from(account: &CompensationProfile) -> Self {
        let cadence = account.cadence();
        Self {
            id: account.id().to_string(),
            tenant_id: account.tenant_id().to_string(),
            employee_id: account.employee_id().to_string(),
            status: account.metadata().status.to_string(),
            created_at: account.metadata().created,
            updated_at: account.metadata().updated,
            amount_units: account.amount().amount_units().to_string(),
            token_symbol: account.amount().token_symbol().to_string(),
            compensation_cadence: cadence.kind().to_string(),
            compensation_cadence_every: cadence.custom_every().map(i32::from),
            compensation_cadence_unit: cadence.custom_unit().map(|unit| unit.to_string()),
            valid_from: account.valid_from(),
            valid_to: account.valid_to(),
        }
    }
}
impl CompensationStore for PgCompensationStore {
    async fn create(
        &self,
        profile: &CompensationProfile,
        audit_event: &AuditEvent,
    ) -> Result<CompensationProfile, CompensationStoreError> {
        let row = CompensationProfileRow::from(profile);
        let mut tx = self.pool.begin().await?;

        let created = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            INSERT INTO compensation_profiles
                (id, tenant_id, employee_id, status, created_at, updated_at, amount_units, token_symbol,
                 compensation_cadence, compensation_cadence_every, compensation_cadence_unit, valid_from, valid_to)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12, $13)
            RETURNING
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.employee_id)
        .bind(&row.status)
        .bind(row.created_at)
        .bind(row.updated_at)
        .bind(&row.amount_units)
        .bind(&row.token_symbol)
        .bind(&row.compensation_cadence)
        .bind(row.compensation_cadence_every)
        .bind(row.compensation_cadence_unit.as_deref())
        .bind(row.valid_from)
        .bind(row.valid_to)
        .fetch_one(&mut *tx)
        .await?;

        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        CompensationProfile::try_from(created)
    }

    async fn update(
        &self,
        profile: &CompensationProfile,
        audit_event: &AuditEvent,
    ) -> Result<CompensationProfile, CompensationStoreError> {
        let row = CompensationProfileRow::from(profile);
        let mut tx = self.pool.begin().await?;

        let updated = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            UPDATE compensation_profiles
            SET amount_units = $4::numeric,
                token_symbol = $5,
                compensation_cadence = $6,
                compensation_cadence_every = $7,
                compensation_cadence_unit = $8,
                valid_from = $9,
                valid_to = $10,
                updated_at = now()
            WHERE id = $1
              AND tenant_id = $2
              AND employee_id = $3
            RETURNING
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.employee_id)
        .bind(&row.amount_units)
        .bind(&row.token_symbol)
        .bind(&row.compensation_cadence)
        .bind(row.compensation_cadence_every)
        .bind(row.compensation_cadence_unit.as_deref())
        .bind(row.valid_from)
        .bind(row.valid_to)
        .fetch_optional(&mut *tx)
        .await?;

        let updated = updated.ok_or(CompensationStoreError::CompensationProfileNotFound)?;
        PgAuditStore::create_in_tx(&mut tx, audit_event).await?;
        tx.commit().await?;

        CompensationProfile::try_from(updated)
    }

    async fn get(
        &self,
        id: &StandardID<IDCompensationProfile>,
    ) -> Result<Option<CompensationProfile>, CompensationStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            SELECT
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            FROM compensation_profiles
            WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(CompensationProfile::try_from).transpose()
    }

    async fn get_active_for_employee(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee_id: &StandardID<IDEmployee>,
    ) -> Result<Option<CompensationProfile>, CompensationStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            SELECT
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            FROM compensation_profiles
            WHERE tenant_id = $1
              AND employee_id = $2
              AND status = 'active'
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(employee_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(CompensationProfile::try_from).transpose()
    }

    async fn list_for_employee(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee_id: &StandardID<IDEmployee>,
    ) -> Result<Vec<CompensationProfile>, CompensationStoreError> {
        let rows = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            SELECT
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            FROM compensation_profiles
            WHERE tenant_id = $1
              AND employee_id = $2
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .bind(employee_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(CompensationProfile::try_from)
            .collect()
    }

    async fn list_active_for_tenant(
        &self,
        tenant_id: &StandardID<IDTenant>,
    ) -> Result<Vec<CompensationProfile>, CompensationStoreError> {
        let rows = sqlx::query_as::<sqlx::Postgres, CompensationProfileRow>(
            r#"
            SELECT
                id,
                tenant_id,
                employee_id,
                status,
                created_at,
                updated_at,
                amount_units::text AS amount_units,
                token_symbol,
                compensation_cadence,
                compensation_cadence_every,
                compensation_cadence_unit,
                valid_from,
                valid_to
            FROM compensation_profiles
            WHERE tenant_id = $1
              AND status = 'active'
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(CompensationProfile::try_from)
            .collect()
    }
}

fn parse_cadence_from_row(
    cadence: &str,
    every: Option<i32>,
    unit: Option<&str>,
) -> Result<CompensationCadence, CompensationStoreError> {
    let cadence = cadence.trim();
    if !cadence.eq_ignore_ascii_case("custom") {
        return CompensationCadence::parse(cadence, None, None).map_err(Into::into);
    }

    let every = every.ok_or(CompensationStoreError::InvalidCadenceEvery)?;
    let every = u16::try_from(every).map_err(|_| CompensationStoreError::InvalidCadenceEvery)?;
    let unit = unit.ok_or(CompensationProfileError::InvalidCustomCadence)?;
    let unit = CadenceUnit::parse(unit)?;

    CompensationCadence::parse(cadence, Some(every), Some(unit)).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditEntityType, AuditEventType};
    use crate::domain::treasury::TokenSymbol;
    use crate::domain::user::IDUser;
    use serde_json::json;

    fn profile() -> CompensationProfile {
        profile_for(StandardID::new(), StandardID::new(), 1_000_000)
    }

    fn profile_for(
        tenant_id: StandardID<IDTenant>,
        employee_id: StandardID<IDEmployee>,
        amount_units: u128,
    ) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id,
            employee_id,
            amount: CompensationAmount::new(amount_units, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn audit_event(profile: &CompensationProfile) -> AuditEvent {
        audit_event_for(profile, AuditEventType::CompensationProfileCreated)
    }

    fn audit_event_for(profile: &CompensationProfile, event_type: AuditEventType) -> AuditEvent {
        AuditEvent::new(
            *profile.tenant_id(),
            StandardID::<IDUser>::new(),
            AuditEntityType::CompensationProfile,
            profile.id().to_string(),
            event_type,
            json!({ "compensation_profile_id": profile.id().to_string() }),
        )
    }

    #[test]
    fn compensation_profile_to_row_roundtrip() {
        let profile = profile();

        let row = CompensationProfileRow::from(&profile);

        assert_eq!(row.id, profile.id().to_string());
        assert_eq!(row.tenant_id, profile.tenant_id().to_string());
        assert_eq!(row.employee_id, profile.employee_id().to_string());
        assert_eq!(row.status, "active");
        assert_eq!(row.amount_units, "1000000");
        assert_eq!(row.token_symbol, "USDC");
        assert_eq!(row.compensation_cadence, "monthly");
        assert_eq!(row.compensation_cadence_every, None);
        assert_eq!(row.compensation_cadence_unit, None);

        let back = CompensationProfile::try_from(row).unwrap();

        assert_eq!(back.id(), profile.id());
        assert_eq!(back.tenant_id(), profile.tenant_id());
        assert_eq!(back.employee_id(), profile.employee_id());
        assert_eq!(back.amount().amount_units(), 1_000_000);
        assert_eq!(back.amount().token_symbol().as_str(), "USDC");
        assert_eq!(back.cadence(), CompensationCadence::Monthly);
    }

    #[sqlx::test]
    async fn creates_compensation_profile_and_audit_event(pool: PgPool) {
        let store = PgCompensationStore::new(pool.clone());
        let profile = profile();
        let event = audit_event(&profile);

        let created = store.create(&profile, &event).await.unwrap();

        assert_eq!(created.id(), profile.id());
        assert_eq!(created.tenant_id(), profile.tenant_id());
        assert_eq!(created.employee_id(), profile.employee_id());
        assert_eq!(created.amount().amount_units(), 1_000_000);

        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(profile.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn gets_compensation_profile_by_id(pool: PgPool) {
        let store = PgCompensationStore::new(pool);
        let profile = profile();

        store
            .create(&profile, &audit_event(&profile))
            .await
            .unwrap();

        let found = store.get(profile.id()).await.unwrap().unwrap();

        assert_eq!(found.id(), profile.id());
        assert_eq!(found.tenant_id(), profile.tenant_id());
        assert_eq!(found.employee_id(), profile.employee_id());
        assert_eq!(found.amount().amount_units(), 1_000_000);
    }

    #[sqlx::test]
    async fn updates_compensation_profile_and_audit_event(pool: PgPool) {
        let store = PgCompensationStore::new(pool.clone());
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let profile = profile_for(tenant_id, employee_id, 1_000_000);

        store
            .create(&profile, &audit_event(&profile))
            .await
            .unwrap();

        let updated_profile = CompensationProfile::restore(
            *profile.id(),
            profile.metadata().clone(),
            CompensationProfileDraft {
                tenant_id,
                employee_id,
                amount: CompensationAmount::new(2_000_000, TokenSymbol::parse("USDC").unwrap())
                    .unwrap(),
                cadence: CompensationCadence::Monthly,
                valid_from: None,
                valid_to: None,
            },
        )
        .unwrap();

        let updated = store
            .update(
                &updated_profile,
                &audit_event_for(&updated_profile, AuditEventType::CompensationProfileUpdated),
            )
            .await
            .unwrap();

        assert_eq!(updated.id(), profile.id());
        assert_eq!(updated.amount().amount_units(), 2_000_000);

        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM audit_events WHERE entity_id = $1")
                .bind(profile.id().to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 2);
    }

    #[sqlx::test]
    async fn get_returns_none_for_unknown_profile(pool: PgPool) {
        let store = PgCompensationStore::new(pool);

        let found = store
            .get(&StandardID::<IDCompensationProfile>::new())
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn gets_active_compensation_profile_for_employee(pool: PgPool) {
        let store = PgCompensationStore::new(pool);
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let active = profile_for(tenant_id, employee_id, 2_000_000);
        let mut inactive = profile_for(tenant_id, employee_id, 1_000_000);
        inactive.deactivate();

        store
            .create(&inactive, &audit_event(&inactive))
            .await
            .unwrap();
        store.create(&active, &audit_event(&active)).await.unwrap();

        let found = store
            .get_active_for_employee(&tenant_id, &employee_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.id(), active.id());
        assert_eq!(found.status(), ObjectStatus::Active);
        assert_eq!(found.amount().amount_units(), 2_000_000);
    }

    #[sqlx::test]
    async fn active_compensation_lookup_returns_none_when_only_inactive_exists(pool: PgPool) {
        let store = PgCompensationStore::new(pool);
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let mut inactive = profile_for(tenant_id, employee_id, 1_000_000);
        inactive.deactivate();

        store
            .create(&inactive, &audit_event(&inactive))
            .await
            .unwrap();

        let found = store
            .get_active_for_employee(&tenant_id, &employee_id)
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn lists_compensation_profiles_for_employee(pool: PgPool) {
        let store = PgCompensationStore::new(pool);
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let other_employee_id = StandardID::new();
        let active = profile_for(tenant_id, employee_id, 2_000_000);
        let mut inactive = profile_for(tenant_id, employee_id, 1_000_000);
        inactive.deactivate();
        let other = profile_for(tenant_id, other_employee_id, 3_000_000);

        store
            .create(&inactive, &audit_event(&inactive))
            .await
            .unwrap();
        store.create(&active, &audit_event(&active)).await.unwrap();
        store.create(&other, &audit_event(&other)).await.unwrap();

        let profiles = store
            .list_for_employee(&tenant_id, &employee_id)
            .await
            .unwrap();

        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().any(|profile| profile.id() == active.id()));
        assert!(profiles.iter().any(|profile| profile.id() == inactive.id()));
        assert!(
            profiles
                .iter()
                .all(|profile| profile.tenant_id() == &tenant_id)
        );
        assert!(
            profiles
                .iter()
                .all(|profile| profile.employee_id() == &employee_id)
        );
    }

    #[sqlx::test]
    async fn lists_active_compensation_profiles_for_tenant(pool: PgPool) {
        let store = PgCompensationStore::new(pool);
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let other_tenant_id = StandardID::new();
        let active = profile_for(tenant_id, employee_id, 2_000_000);
        let mut inactive = profile_for(tenant_id, StandardID::new(), 1_000_000);
        inactive.deactivate();
        let other_tenant = profile_for(other_tenant_id, StandardID::new(), 3_000_000);

        store
            .create(&inactive, &audit_event(&inactive))
            .await
            .unwrap();
        store.create(&active, &audit_event(&active)).await.unwrap();
        store
            .create(&other_tenant, &audit_event(&other_tenant))
            .await
            .unwrap();

        let profiles = store.list_active_for_tenant(&tenant_id).await.unwrap();

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id(), active.id());
        assert_eq!(profiles[0].tenant_id(), &tenant_id);
        assert_eq!(profiles[0].status(), ObjectStatus::Active);
    }
}
