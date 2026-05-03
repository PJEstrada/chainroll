use crate::domain::audit::{
    AuditEntityType, AuditEvent, AuditEventType, IDAudit, ParseAuditEntityTypeError,
    ParseAuditEventTypeError,
};
use crate::domain::ids::{IdError, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::user::IDUser;
use crate::services::datastore::AuditStore;
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

#[derive(sqlx::FromRow)]
struct AuditEventRow {
    id: String,
    tenant_id: String,
    actor_id: String,
    entity_type: String,
    entity_id: String,
    event_type: String,
    #[sqlx(json)]
    payload: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<AuditEventRow> for AuditEvent {
    type Error = AuditStoreError;

    fn try_from(row: AuditEventRow) -> Result<Self, Self::Error> {
        Ok(AuditEvent::new(
            StandardID::<IDTenant>::from_str(row.tenant_id.as_str())?,
            StandardID::<IDUser>::from_str(row.actor_id.as_str())?,
            AuditEntityType::from_str(row.entity_type.as_str())?,
            row.entity_id,
            AuditEventType::from_str(row.event_type.as_str())?,
            row.payload,
        )
        .with_id(StandardID::<IDAudit>::from_str(row.id.as_str())?)
        .with_created_at(row.created_at))
    }
}

impl From<&AuditEvent> for AuditEventRow {
    fn from(event: &AuditEvent) -> Self {
        Self {
            id: event.id().to_string(),
            tenant_id: event.tenant_id().to_string(),
            actor_id: event.actor_id().to_string(),
            entity_type: event.entity_type().to_string(),
            entity_id: event.entity_id().to_string(),
            event_type: event.event_type().to_string(),
            payload: event.payload().clone(),
            created_at: event.created_at().to_owned(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AuditStoreError {
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid audit entity type: {0}")]
    InvalidEntityType(#[from] ParseAuditEntityTypeError),
    #[error("Invalid audit event type: {0}")]
    InvalidEventType(#[from] ParseAuditEventTypeError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct PgAuditStore {
    pool: PgPool,
}

impl PgAuditStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn create_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event: &AuditEvent,
    ) -> Result<AuditEvent, AuditStoreError> {
        let row = AuditEventRow::from(event);

        let created = sqlx::query_as::<sqlx::Postgres, AuditEventRow>(
            r#"
            INSERT INTO audit_events
                (id, tenant_id, actor_id, entity_type, entity_id, event_type, payload, created_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.actor_id)
        .bind(&row.entity_type)
        .bind(&row.entity_id)
        .bind(&row.event_type)
        .bind(sqlx::types::Json(&row.payload))
        .bind(row.created_at)
        .fetch_one(&mut **tx)
        .await?;

        AuditEvent::try_from(created)
    }
}

impl AuditStore for PgAuditStore {
    async fn create(&self, event: &AuditEvent) -> Result<AuditEvent, AuditStoreError> {
        let mut tx = self.pool.begin().await?;
        let created = Self::create_in_tx(&mut tx, event).await?;
        tx.commit().await?;

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_event() -> AuditEvent {
        AuditEvent::new(
            StandardID::<IDTenant>::new(),
            StandardID::<IDUser>::new(),
            AuditEntityType::Employee,
            "000000000003V",
            AuditEventType::EmployeeCreated,
            json!({
                "identifier": "EMP-001",
                "first_name": "Jane"
            }),
        )
    }

    #[test]
    fn test_audit_event_to_row_roundtrip() {
        let event = test_event();

        let row = AuditEventRow::from(&event);
        assert_eq!(row.id, event.id().to_string());
        assert_eq!(row.tenant_id, event.tenant_id().to_string());
        assert_eq!(row.actor_id, event.actor_id().to_string());
        assert_eq!(row.entity_type, "employee");
        assert_eq!(row.entity_id, "000000000003V");
        assert_eq!(row.event_type, "employee_created");

        let back = AuditEvent::try_from(row).unwrap();
        assert_eq!(back.id(), event.id());
        assert_eq!(back.tenant_id(), event.tenant_id());
        assert_eq!(back.actor_id(), event.actor_id());
        assert_eq!(back.entity_type(), event.entity_type());
        assert_eq!(back.entity_id(), event.entity_id());
        assert_eq!(back.event_type(), event.event_type());
        assert_eq!(back.payload(), event.payload());
    }

    #[sqlx::test]
    async fn test_create_audit_event(pool: PgPool) {
        let store = PgAuditStore::new(pool);
        let event = test_event();

        let created = store.create(&event).await.unwrap();

        assert_eq!(created.id(), event.id());
        assert_eq!(created.tenant_id(), event.tenant_id());
        assert_eq!(created.actor_id(), event.actor_id());
        assert_eq!(created.entity_type(), AuditEntityType::Employee);
        assert_eq!(created.entity_id(), "000000000003V");
        assert_eq!(created.event_type(), AuditEventType::EmployeeCreated);
        assert_eq!(created.payload()["identifier"], "EMP-001");
    }
}
