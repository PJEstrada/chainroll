use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateBatchRequest;
pub struct CreateBatchResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: CreateBatchRequest,
) -> Result<CreateBatchResponse> {
    unimplemented!()
}
