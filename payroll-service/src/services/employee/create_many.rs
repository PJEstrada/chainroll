use crate::Result;
use crate::domain::employee::Employee;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateManyRequest {
    pub tenant_id: IDTenant,
    pub employees: Vec<Employee>,
}

pub struct CreateManyResponse {
    pub employees: Vec<Employee>,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: CreateManyRequest,
) -> Result<CreateManyResponse> {
    unimplemented!()
}
