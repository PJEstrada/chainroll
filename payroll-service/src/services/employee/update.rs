use crate::Result;
use crate::domain::division::IDDivision;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::datastore::postgres::employee_store::EmployeeStoreError;
use crate::services::employee::service::EmployeeServiceImpl;
use serde::Deserialize;
use serde_json::Value;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[serde_as]
#[derive(Deserialize)]
pub struct UpdateEmployeeData {
    pub id: StandardID<IDEmployee>,
    pub identifier: String,
    pub first_name: String,
    pub last_name: String,
    pub divisions: Option<Vec<StandardID<IDDivision>>>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub culture: Option<LanguageIdentifier>,
    pub attributes: Option<HashMap<String, Value>>,
}
pub struct UpdateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub data: UpdateEmployeeData,
}
#[derive(Debug)]
pub struct UpdateResponse {
    pub employee: Employee,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: UpdateRequest,
) -> Result<UpdateResponse> {
    let employee = Employee::new(req.data.identifier, req.data.first_name, req.data.last_name)
        .with_id(req.data.id)
        .with_divisions(req.data.divisions.unwrap_or_default())
        .with_culture(req.data.culture)
        .with_attributes(req.data.attributes);

    let employee = svc
        .store()
        .update(&req.tenant_id, &employee)
        .await
        .map_err(|err| match err {
            EmployeeStoreError::EmployeeNotFound => {
                error_stack::Report::new(crate::error::Error::NotFound)
            }
            err => error_stack::Report::new(err).change_context(crate::error::Error::Database),
        })?;

    Ok(UpdateResponse { employee })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockEmployeeStore;

    fn make_request() -> UpdateRequest {
        UpdateRequest {
            tenant_id: StandardID::new(),
            data: UpdateEmployeeData {
                id: StandardID::new(),
                identifier: "EMP-002".into(),
                first_name: "Jane".into(),
                last_name: "Smith".into(),
                divisions: None,
                culture: None,
                attributes: None,
            },
        }
    }

    #[tokio::test]
    async fn returns_updated_employee() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_update()
            .returning(|_, employee| Ok(employee.clone()));
        let svc = EmployeeServiceImpl::new(store);

        let response = execute(&svc, make_request()).await.unwrap();

        assert_eq!(response.employee.identifier(), "EMP-002");
        assert_eq!(response.employee.first_name(), "Jane");
        assert_eq!(response.employee.last_name(), "Smith");
    }

    #[tokio::test]
    async fn maps_missing_employee_to_not_found() {
        let mut store = MockEmployeeStore::new();
        store
            .expect_update()
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
