use crate::Result;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct UpdateRequest {
    pub tenant_id: IDTenant,
    pub id: IDEmployee,
    pub updated: Employee,
}

pub struct UpdateResponse {
    pub employee: Employee,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: UpdateRequest,
) -> Result<UpdateResponse> {
    unimplemented!()
}
