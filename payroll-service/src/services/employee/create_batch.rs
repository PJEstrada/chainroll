use crate::Result;
use crate::domain::employee::Employee;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateBatchRequest {
    pub tenant_id: IDTenant,
    pub employees: Vec<Employee>,
}

pub struct CreateBatchResponse;

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: CreateBatchRequest,
) -> Result<CreateBatchResponse> {
    unimplemented!()
}
