use crate::Result;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::datastore::postgres::employee_store::EmployeeStoreError;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct DeleteRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub id: StandardID<IDEmployee>,
}

#[derive(Debug)]
pub struct DeleteResponse;

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: DeleteRequest,
) -> Result<DeleteResponse> {
    svc.store()
        .delete(&req.tenant_id, &req.id)
        .await
        .map_err(|err| match err {
            EmployeeStoreError::EmployeeNotFound => {
                error_stack::Report::new(crate::error::Error::NotFound)
            }
            err => error_stack::Report::new(err).change_context(crate::error::Error::Database),
        })?;

    Ok(DeleteResponse)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;

    fn make_request() -> DeleteRequest {
        DeleteRequest {
            tenant_id: StandardID::new(),
            id: StandardID::new(),
        }
    }

    #[tokio::test]
    async fn deletes_employee() {
        let mut store = MockEmployeeStore::new();
        store.expect_delete().returning(|_, _| Ok(()));
        let svc = EmployeeServiceImpl::new(store);

        execute(&svc, make_request()).await.unwrap();
    }

    #[tokio::test]
    async fn maps_missing_employee_to_not_found() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_delete()
            .returning(|_, _| Err(EmployeeStoreError::EmployeeNotFound));
        let svc = EmployeeServiceImpl::new(store);

        let result = execute(&svc, make_request()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().current_context(),
            crate::error::Error::NotFound
        ));
    }
}
