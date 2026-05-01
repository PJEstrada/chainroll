use crate::Result;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;
use error_stack::ResultExt;

pub struct ExistsRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub id: StandardID<IDEmployee>,
}

#[derive(Debug)]
pub struct ExistsResponse {
    pub exists: bool,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: ExistsRequest,
) -> Result<ExistsResponse> {
    let exists = svc
        .store()
        .exists(&req.tenant_id, &req.id)
        .await
        .change_context(crate::error::Error::Database)?;

    Ok(ExistsResponse { exists })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;
    use crate::services::datastore::postgres::employee_store::EmployeeStoreError;

    fn make_request() -> ExistsRequest {
        ExistsRequest {
            tenant_id: StandardID::new(),
            id: StandardID::new(),
        }
    }

    #[tokio::test]
    async fn returns_exists_from_store() {
        let mut store = MockEmployeeStore::new();
        store.expect_exists().returning(|_, _| Ok(true));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        assert!(response.exists);
    }

    #[tokio::test]
    async fn returns_database_error_when_store_fails() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_exists()
            .returning(|_, _| Err(EmployeeStoreError::Database(sqlx::Error::RowNotFound)));
        let svc = EmployeeServiceImpl::new(store);

        let result = execute(&svc, make_request()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().current_context(),
            crate::error::Error::Database
        ));
    }
}
