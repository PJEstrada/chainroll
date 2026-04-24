use crate::domain::division::IDDivision;
use crate::domain::employee::Employee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;
use crate::Result;
use error_stack::ResultExt;
use serde::Deserialize;
use serde_json::Value;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[serde_as]
#[derive(Deserialize)]
pub struct CreateEmployeeData {
    pub identifier: String,
    pub first_name: String,
    pub last_name: String,
    pub divisions: Option<Vec<StandardID<IDDivision>>>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub culture: Option<LanguageIdentifier>,
    pub attributes: Option<HashMap<String, Value>>,
}

pub struct CreateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub data: CreateEmployeeData,
}

pub struct CreateResponse {
    pub employee: Employee,
}

pub(super) async fn execute<S: EmployeeStore>(
    svc: &EmployeeServiceImpl<S>,
    req: CreateRequest,
) -> Result<CreateResponse> {
    let employee = Employee::new(req.data.identifier, req.data.first_name, req.data.last_name)
        .with_divisions(req.data.divisions.unwrap_or_default())
        .with_culture(req.data.culture)
        .with_attributes(req.data.attributes);

    let employee = svc
        .store()
        .create(&req.tenant_id, &employee)
        .await
        .change_context(crate::error::Error::Database)?;

    Ok(CreateResponse { employee })
}
