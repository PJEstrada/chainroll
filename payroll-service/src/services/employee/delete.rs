use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct DeleteRequest;
pub struct DeleteResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: DeleteRequest,
) -> Result<DeleteResponse> {
    unimplemented!()
}
