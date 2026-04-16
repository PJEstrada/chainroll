use crate::Result;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct GetRequest {
    pub tenant_id: IDTenant,
    pub id: IDEmployee,
}

pub struct GetResponse {
    pub employee: Option<Employee>,
}

pub(super) async fn execute(_svc: &EmployeeServiceImpl, _req: GetRequest) -> Result<GetResponse> {
    unimplemented!()
}
