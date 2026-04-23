use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct ListRequest;
pub struct ListResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: ListRequest,
) -> Result<ListResponse> {
    unimplemented!()
}
