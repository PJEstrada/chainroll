use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct ExistsRequest;
pub struct ExistsResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: ExistsRequest,
) -> Result<ExistsResponse> {
    unimplemented!()
}
