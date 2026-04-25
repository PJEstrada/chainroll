pub mod postgres;

use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use postgres::employee_store::EmployeeStoreError;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait EmployeeStore {
    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> std::result::Result<Option<Employee>, EmployeeStoreError>;

    async fn create(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee: &Employee,
    ) -> std::result::Result<Employee, EmployeeStoreError>;

    async fn update(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee: &Employee,
    ) -> Result<Employee, EmployeeStoreError>;
}
