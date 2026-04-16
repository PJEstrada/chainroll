use crate::Result;
use crate::domain::employee::{Employee, EmployeeQuery};
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct ListRequest {
    pub tenant_id: IDTenant,
    pub query: EmployeeQuery,
}

pub struct ListResponse {
    pub employees: Vec<Employee>,
}

pub(super) async fn execute(_svc: &EmployeeServiceImpl, _req: ListRequest) -> Result<ListResponse> {
    unimplemented!()
}
