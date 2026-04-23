use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus, ParseStatusError};
use crate::domain::division::IDDivision;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::{IdError, StandardID};
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct EmployeeRow {
    id: String,
    tenant_id: String,
    identifier: String,
    first_name: String,
    last_name: String,
    #[sqlx(json)]
    divisions: Vec<String>,
    culture: Option<String>,
    #[sqlx(json)]
    attributes: Option<HashMap<String, serde_json::Value>>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<EmployeeRow> for Employee {
    type Error = EmployeeStoreError;

    fn try_from(row: EmployeeRow) -> std::result::Result<Self, EmployeeStoreError> {
        let id = StandardID::from_str(row.id.as_str())?;

        let divisions = row
            .divisions
            .iter()
            .map(|s| Ok(StandardID::<IDDivision>::from_str(s)?))
            .collect::<std::result::Result<Vec<_>, EmployeeStoreError>>()?;

        let culture = row
            .culture
            .map(|s| unic_langid::LanguageIdentifier::from_str(s.as_str()))
            .transpose()?;

        let status = ObjectStatus::from_str(row.status.as_str())?;
        let metadata = LifecycleMeta {
            status,
            created: row.created_at,
            updated: row.updated_at,
        };

        Ok(Employee::new(row.identifier, row.first_name, row.last_name)
            .with_id(id)
            .with_metadata(metadata)
            .with_culture(culture)
            .with_divisions(divisions)
            .with_attributes(row.attributes))
    }
}

#[derive(Debug, Error)]
pub enum EmployeeStoreError {
    #[error("Employee already exists")]
    EmployeeAlreadyExists,
    #[error("Employee not found")]
    EmployeeNotFound,
    #[error("Invalid ID: {0}")]
    InvalidId(#[from] IdError),
    #[error("Invalid locale: {0}")]
    InvalidLocale(#[from] unic_langid::LanguageIdentifierError),
    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] ParseStatusError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct PgEmployeeStore {
    pool: PgPool,
}

impl PgEmployeeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl EmployeeStore for PgEmployeeStore {
    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> std::result::Result<Option<Employee>, EmployeeStoreError> {
        let row = sqlx::query_as::<sqlx::Postgres, EmployeeRow>(
            "SELECT * FROM employees WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Employee::try_from).transpose()
    }
}
