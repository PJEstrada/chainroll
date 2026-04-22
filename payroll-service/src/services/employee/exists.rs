use crate::Result;
use crate::domain::employee::IDEmployee;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct ExistsRequest {
    pub tenant_id: IDTenant,
    pub id: IDEmployee,
}

pub struct ExistsResponse {
    pub exists: bool,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: ExistsRequest,
) -> Result<ExistsResponse> {
    unimplemented!()
}
