use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::service::EmployeeServiceImpl;

pub struct CreateManyRequest;
pub struct CreateManyResponse;

pub(super) async fn execute<S: EmployeeStore>(
    _svc: &EmployeeServiceImpl<S>,
    _req: CreateManyRequest,
) -> Result<CreateManyResponse> {
    unimplemented!()
}
