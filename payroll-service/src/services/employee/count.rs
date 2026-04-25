use crate::Result;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;
use error_stack::ResultExt;

pub struct CountRequest {
    pub tenant_id: StandardID<IDTenant>,
}

#[derive(Debug)]
pub struct CountResponse {
    pub count: i64,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: CountRequest,
) -> Result<CountResponse> {
    let count = svc
        .store()
        .count(&req.tenant_id)
        .await
        .change_context(crate::error::Error::Database)?;

    Ok(CountResponse { count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;
    use crate::services::datastore::postgres::employee_store::EmployeeStoreError;

    fn make_request() -> CountRequest {
        CountRequest {
            tenant_id: StandardID::new(),
        }
    }

    #[tokio::test]
    async fn returns_count_from_store() {
        let mut store = MockEmployeeStore::new();
        store.expect_count().returning(|_| Ok(3));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        assert_eq!(response.count, 3);
    }

    #[tokio::test]
    async fn returns_database_error_when_store_fails() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_count()
            .returning(|_| Err(EmployeeStoreError::Database(sqlx::Error::RowNotFound)));
        let svc = EmployeeServiceImpl::new(store);

        let result = execute(&svc, make_request()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().current_context(),
            crate::error::Error::Database
        ));
    }
}
