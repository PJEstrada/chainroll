use crate::Result;
use crate::domain::employee::IDEmployee;
use crate::domain::tenant::IDTenant;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct DeleteRequest {
    pub tenant_id: IDTenant,
    pub id: IDEmployee,
}

pub struct DeleteResponse {
    pub deleted: bool,
}

pub(super) async fn execute(
    _svc: &EmployeeServiceImpl,
    _req: DeleteRequest,
) -> Result<DeleteResponse> {
    unimplemented!()
}
