use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateRequest;
pub struct CreateResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: CreateRequest,
) -> Result<CreateResponse> {
    unimplemented!()
}
