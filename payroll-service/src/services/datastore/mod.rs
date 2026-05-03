pub mod postgres;

use crate::domain::audit::AuditEvent;
use crate::domain::compensation::{CompensationProfile, IDCompensationProfile};
use crate::domain::employee::{Employee, EmployeeQuery, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{IDTreasuryAccount, TreasuryAccount, TreasuryAccountQuery};
use crate::services::datastore::postgres::compensation_store::CompensationStoreError;
use postgres::audit_store::AuditStoreError;
use postgres::employee_store::EmployeeStoreError;
use postgres::treasury_store::TreasuryStoreError;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait AuditStore {
    async fn create(&self, event: &AuditEvent) -> std::result::Result<AuditEvent, AuditStoreError>;
}

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

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait TreasuryStore {
    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDTreasuryAccount>,
    ) -> std::result::Result<Option<TreasuryAccount>, TreasuryStoreError>;

    async fn list(
        &self,
        tenant_id: &StandardID<IDTenant>,
        query: &TreasuryAccountQuery,
    ) -> std::result::Result<Vec<TreasuryAccount>, TreasuryStoreError>;

    async fn create(
        &self,
        account: &TreasuryAccount,
        audit_event: &AuditEvent,
    ) -> std::result::Result<TreasuryAccount, TreasuryStoreError>;

    async fn update(
        &self,
        account: &TreasuryAccount,
        audit_event: &AuditEvent,
    ) -> std::result::Result<TreasuryAccount, TreasuryStoreError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait CompensationStore: Send + Sync + 'static {
    async fn create(
        &self,
        profile: &CompensationProfile,
        audit_event: &AuditEvent,
    ) -> Result<CompensationProfile, CompensationStoreError>;

    async fn update(
        &self,
        profile: &CompensationProfile,
        audit_event: &AuditEvent,
    ) -> Result<CompensationProfile, CompensationStoreError>;

    async fn get(
        &self,
        id: &StandardID<IDCompensationProfile>,
    ) -> Result<Option<CompensationProfile>, CompensationStoreError>;

    async fn get_active_for_employee(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee_id: &StandardID<IDEmployee>,
    ) -> Result<Option<CompensationProfile>, CompensationStoreError>;

    async fn list_for_employee(
        &self,
        tenant_id: &StandardID<IDTenant>,
        employee_id: &StandardID<IDEmployee>,
    ) -> Result<Vec<CompensationProfile>, CompensationStoreError>;
}
