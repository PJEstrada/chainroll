use crate::Result;
use crate::domain::employee::Employee;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateRequest {
    pub tenant_id: IDTenant,
    pub employee: Employee,
}

pub struct CreateResponse {
    pub employee: Employee,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: CreateRequest,
) -> Result<CreateResponse> {
    unimplemented!()
}
