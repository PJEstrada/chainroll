pub mod postgres;

use crate::domain::employee::{Employee, EmployeeQuery, IDEmployee};
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

    async fn list(
        &self,
        tenant_id: &StandardID<IDTenant>,
        query: &EmployeeQuery,
    ) -> std::result::Result<Vec<Employee>, EmployeeStoreError>;

    async fn count(
        &self,
        tenant_id: &StandardID<IDTenant>,
    ) -> std::result::Result<i64, EmployeeStoreError>;

    async fn exists(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> std::result::Result<bool, EmployeeStoreError>;

    async fn delete(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDEmployee>,
    ) -> std::result::Result<(), EmployeeStoreError>;
}
