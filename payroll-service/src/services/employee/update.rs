use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct UpdateRequest;
pub struct UpdateResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: UpdateRequest,
) -> Result<UpdateResponse> {
    unimplemented!()
}
