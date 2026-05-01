use crate::Result;
use crate::domain::employee::{Employee, EmployeeQuery};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;
use error_stack::ResultExt;

pub struct ListRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub query: EmployeeQuery,
}

#[derive(Debug)]
pub struct ListResponse {
    pub employees: Vec<Employee>,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: ListRequest,
) -> Result<ListResponse> {
    let employees = svc
        .store()
        .list(&req.tenant_id, &req.query)
        .await
        .change_context(crate::error::Error::Database)?;

    Ok(ListResponse { employees })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;
    use crate::services::datastore::postgres::employee_store::EmployeeStoreError;

    fn make_request() -> ListRequest {
        ListRequest {
            tenant_id: StandardID::new(),
            query: EmployeeQuery::default(),
        }
    }

    #[tokio::test]
    async fn returns_employees_from_store() {
        let employee = Employee::new("EMP-001".into(), "Jane".into(), "Doe".into());
        let mut store = MockEmployeeStore::new();
        store
            .expect_list()
            .returning(move |_, _| Ok(vec![employee.clone()]));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        assert_eq!(response.employees.len(), 1);
        assert_eq!(response.employees[0].identifier(), "EMP-001");
    }

    #[tokio::test]
    async fn returns_database_error_when_store_fails() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_list()
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
