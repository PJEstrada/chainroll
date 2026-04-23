use crate::Result;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;
use error_stack::ResultExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct GetRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub id: StandardID<IDEmployee>,
}
#[derive(Debug, Serialize)]
pub struct GetResponse {
    pub employee: Option<Employee>,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: GetRequest,
) -> Result<GetResponse> {
    let employee = svc
        .store()
        .get(&req.tenant_id, &req.id)
        .await
        .change_context(crate::error::Error::Database)?;

    Ok(GetResponse { employee })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;
    use crate::services::datastore::postgres::employee_store::EmployeeStoreError;

    fn make_request() -> GetRequest {
        GetRequest {
            tenant_id: StandardID::new(),
            id: StandardID::new(),
        }
    }

    #[tokio::test]
    async fn returns_employee_when_store_finds_one() {
        let employee = Employee::new("EMP-001".into(), "Jane".into(), "Doe".into());
        let mut store = MockEmployeeStore::new();
        store
            .expect_get()
            .returning(move |_, _| Ok(Some(employee.clone())));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        let emp = response.employee.expect("should contain an employee");
        let json = serde_json::to_value(&emp).unwrap();
        assert_eq!(json["identifier"], "EMP-001");
        assert_eq!(json["first_name"], "Jane");
        assert_eq!(json["last_name"], "Doe");
    }

    #[tokio::test]
    async fn returns_none_when_store_finds_nothing() {
        let mut store = MockEmployeeStore::new();
        store.expect_get().returning(|_, _| Ok(None));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        assert!(response.employee.is_none());
    }

    #[tokio::test]
    async fn returns_database_error_when_store_fails() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_get()
            .returning(|_, _| Err(EmployeeStoreError::Database(sqlx::Error::RowNotFound)));
        let svc = EmployeeServiceImpl::new(store);

        let result = execute(&svc, make_request()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err.current_context(), crate::error::Error::Database),
            "expected Error::Database, got: {:?}",
            err.current_context()
        );
    }
}
