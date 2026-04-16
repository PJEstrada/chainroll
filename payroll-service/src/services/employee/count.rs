use crate::Result;
use crate::domain::employee::EmployeeQuery;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CountRequest {
    pub tenant_id: IDTenant,
    pub query: EmployeeQuery,
}

pub struct CountResponse {
    pub count: i64,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: CountRequest,
) -> Result<CountResponse> {
    unimplemented!()
}
