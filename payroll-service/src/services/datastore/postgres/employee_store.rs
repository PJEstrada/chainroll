use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus, ParseStatusError};
use crate::domain::division::IDDivision;
use crate::domain::employee::{Employee, EmployeeQuery, IDEmployee};
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

impl From<(&StandardID<IDTenant>, &Employee)> for EmployeeRow {
    fn from((tenant_id, employee): (&StandardID<IDTenant>, &Employee)) -> Self {
        Self {
            id: employee.id().to_string(),
            tenant_id: tenant_id.to_string(),
            identifier: employee.identifier().to_string(),
            first_name: employee.first_name().to_string(),
            last_name: employee.last_name().to_string(),
            divisions: employee.divisions().iter().map(|d| d.to_string()).collect(),
            culture: employee.culture().clone().map(|c| c.to_string()),
            attributes: employee.attributes().clone(),
            status: employee.metadata().status.to_string(),
            created_at: employee.metadata().created,
            updated_at: employee.metadata().updated,
        }
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

fn query_limit(query: &EmployeeQuery) -> std::result::Result<i64, EmployeeStoreError> {
    i64::try_from(query.base.limit.unwrap_or(100)).map_err(|err| anyhow::anyhow!(err).into())
}

fn query_offset(query: &EmployeeQuery) -> std::result::Result<i64, EmployeeStoreError> {
    i64::try_from(query.base.offset.unwrap_or(0)).map_err(|err| anyhow::anyhow!(err).into())
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

    async fn create(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee: &Employee,
    ) -> Result<Employee, EmployeeStoreError> {
        let row = EmployeeRow::from((tenant_id, employee));

        let result = sqlx::query_as::<sqlx::Postgres, EmployeeRow>(
            r#"
        INSERT INTO employees (id, tenant_id, identifier, first_name, last_name, divisions, culture, attributes, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
        )
            .bind(&row.id)
            .bind(&row.tenant_id)
            .bind(&row.identifier)
            .bind(&row.first_name)
            .bind(&row.last_name)
            .bind(sqlx::types::Json(&row.divisions))
            .bind(&row.culture)
            .bind(sqlx::types::Json(&row.attributes))
            .bind(&row.status)
            .fetch_one(&self.pool)
            .await?;

        Employee::try_from(result)
    }

    async fn update(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee: &Employee,
    ) -> Result<Employee, EmployeeStoreError> {
        let row = EmployeeRow::from((tenant_id, employee));

        let result = sqlx::query_as::<sqlx::Postgres, EmployeeRow>(
            r#"
        UPDATE employees
        SET identifier = $3,
            first_name = $4,
            last_name = $5,
            divisions = $6,
            culture = $7,
            attributes = $8,
            status = $9,
            updated_at = now()
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#,
        )
        .bind(&row.id)
        .bind(&row.tenant_id)
        .bind(&row.identifier)
        .bind(&row.first_name)
        .bind(&row.last_name)
        .bind(sqlx::types::Json(&row.divisions))
        .bind(&row.culture)
        .bind(sqlx::types::Json(&row.attributes))
        .bind(&row.status)
        .fetch_optional(&self.pool)
        .await?;

        result
            .map(Employee::try_from)
            .transpose()?
            .ok_or(EmployeeStoreError::EmployeeNotFound)
    }

    async fn list(
        &self,
        tenant_id: &StandardID<IDTenant>,
        query: &EmployeeQuery,
    ) -> Result<Vec<Employee>, EmployeeStoreError> {
        let limit = query_limit(query)?;
        let offset = query_offset(query)?;

        let rows = if let Some(division_id) = &query.division_id {
            sqlx::query_as::<sqlx::Postgres, EmployeeRow>(
                r#"
                SELECT * FROM employees
                WHERE tenant_id = $1 AND divisions ? $2
                ORDER BY created_at ASC, id ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id.to_string())
            .bind(division_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<sqlx::Postgres, EmployeeRow>(
                r#"
                SELECT * FROM employees
                WHERE tenant_id = $1
                ORDER BY created_at ASC, id ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tenant_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(Employee::try_from).collect()
    }

    async fn count(&self, tenant_id: &StandardID<IDTenant>) -> Result<i64, EmployeeStoreError> {
        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM employees WHERE tenant_id = $1")
                .bind(tenant_id.to_string())
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    async fn exists(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> Result<bool, EmployeeStoreError> {
        let (exists,) = sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS(SELECT 1 FROM employees WHERE id = $1 AND tenant_id = $2)",
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn delete(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> Result<(), EmployeeStoreError> {
        let result = sqlx::query("DELETE FROM employees WHERE id = $1 AND tenant_id = $2")
            .bind(id.to_string())
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(EmployeeStoreError::EmployeeNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_to_row_roundtrip() {
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        // domain → row
        let row = EmployeeRow::from((&tenant_id, &employee));

        assert_eq!(row.identifier, "12345");
        assert_eq!(row.first_name, "John");
        assert_eq!(row.last_name, "Doe");
        assert_eq!(row.tenant_id, tenant_id.to_string());
        assert_eq!(row.status, "active"); // whatever the default is
        assert!(row.divisions.is_empty());

        // row → domain (roundtrip)
        let back = Employee::try_from(row).unwrap();
        assert_eq!(back.identifier(), employee.identifier());
        assert_eq!(back.first_name(), employee.first_name());
    }

    #[sqlx::test]
    async fn test_create_employee(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        let result = store.create(&tenant_id, &employee).await.unwrap();
        assert_eq!(result.identifier(), "12345");
        assert_eq!(result.first_name(), "John");
        assert_eq!(result.last_name(), "Doe");
    }

    #[sqlx::test]
    async fn test_get_employee(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        let result = store.create(&tenant_id, &employee).await.unwrap();
        assert_eq!(result.identifier(), "12345");
        assert_eq!(result.first_name(), "John");
        assert_eq!(result.last_name(), "Doe");

        let result2 = store.get(&tenant_id, result.id()).await.unwrap();
        assert_eq!(result.identifier(), result2.clone().unwrap().identifier());
        assert_eq!(result.first_name(), result2.clone().unwrap().first_name());
        assert_eq!(result.last_name(), result2.unwrap().last_name());
    }

    #[sqlx::test]
    async fn test_list_employees(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee1 = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());
        let employee2 = Employee::new("67890".to_string(), "Jane".to_string(), "Smith".to_string());

        store.create(&tenant_id, &employee1).await.unwrap();
        store.create(&tenant_id, &employee2).await.unwrap();

        let result = store
            .list(&tenant_id, &EmployeeQuery::default())
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].identifier(), "12345");
        assert_eq!(result[1].identifier(), "67890");
    }

    #[sqlx::test]
    async fn test_count_employees(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let other_tenant_id = StandardID::<IDTenant>::new();
        let employee1 = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());
        let employee2 = Employee::new("67890".to_string(), "Jane".to_string(), "Smith".to_string());
        let employee3 = Employee::new(
            "ABCDE".to_string(),
            "Other".to_string(),
            "Tenant".to_string(),
        );

        store.create(&tenant_id, &employee1).await.unwrap();
        store.create(&tenant_id, &employee2).await.unwrap();
        store.create(&other_tenant_id, &employee3).await.unwrap();

        assert_eq!(store.count(&tenant_id).await.unwrap(), 2);
    }

    #[sqlx::test]
    async fn test_exists_employee(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        let created = store.create(&tenant_id, &employee).await.unwrap();

        assert!(store.exists(&tenant_id, created.id()).await.unwrap());
        assert!(!store.exists(&tenant_id, &StandardID::new()).await.unwrap());
    }

    #[sqlx::test]
    async fn test_delete_employee(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        let created = store.create(&tenant_id, &employee).await.unwrap();

        store.delete(&tenant_id, created.id()).await.unwrap();
        assert!(!store.exists(&tenant_id, created.id()).await.unwrap());
    }

    #[sqlx::test]
    async fn test_update_employee(pool: PgPool) {
        let store = PgEmployeeStore::new(pool);
        let tenant_id = StandardID::<IDTenant>::new();
        let employee = Employee::new("12345".to_string(), "John".to_string(), "Doe".to_string());

        let created = store.create(&tenant_id, &employee).await.unwrap();
        let updated = Employee::new("67890".to_string(), "Jane".to_string(), "Smith".to_string())
            .with_id(*created.id());

        let result = store.update(&tenant_id, &updated).await.unwrap();

        assert_eq!(result.id(), created.id());
        assert_eq!(result.identifier(), "67890");
        assert_eq!(result.first_name(), "Jane");
        assert_eq!(result.last_name(), "Smith");
    }
}
