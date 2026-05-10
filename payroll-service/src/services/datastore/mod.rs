pub mod postgres;

use crate::domain::audit::AuditEvent;
use crate::domain::compensation::{CompensationProfile, IDCompensationProfile};
use crate::domain::employee::{Employee, EmployeeQuery, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::payout_attempt::{IDPayoutAttempt, PayoutAttempt};
use crate::domain::payout_instruction::{IDPayoutInstruction, PayoutInstruction};
use crate::domain::payrun::{IDPayrun, Payrun};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{IDTreasuryAccount, TreasuryAccount, TreasuryAccountQuery};
use crate::services::datastore::postgres::compensation_store::CompensationStoreError;
use crate::services::datastore::postgres::payout_attempt_store::PayoutAttemptStoreError;
use crate::services::datastore::postgres::payout_instruction_store::PayoutInstructionStoreError;
use crate::services::datastore::postgres::payrun_store::PayrunStoreError;
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

    async fn list_active(
        &self,
        tenant_id: &StandardID<IDTenant>,
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

    async fn list_default_active(
        &self,
        tenant_id: &StandardID<IDTenant>,
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

    async fn list_active_for_tenant(
        &self,
        tenant_id: &StandardID<IDTenant>,
    ) -> Result<Vec<CompensationProfile>, CompensationStoreError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayrunStore: Send + Sync + 'static {
    async fn create(
        &self,
        payrun: &Payrun,
        audit_event: &AuditEvent,
    ) -> Result<Payrun, PayrunStoreError>;

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayrun>,
    ) -> Result<Option<Payrun>, PayrunStoreError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayoutInstructionStore: Send + Sync + 'static {
    async fn create_many_idempotent(
        &self,
        instructions: &[PayoutInstruction],
        audit_events: &[AuditEvent],
    ) -> Result<Vec<PayoutInstruction>, PayoutInstructionStoreError>;

    async fn list_for_payrun(
        &self,
        tenant_id: &StandardID<IDTenant>,
        payrun_id: &StandardID<IDPayrun>,
    ) -> Result<Vec<PayoutInstruction>, PayoutInstructionStoreError>;

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayoutInstruction>,
    ) -> Result<Option<PayoutInstruction>, PayoutInstructionStoreError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayoutAttemptStore: Send + Sync + 'static {
    async fn create_started(
        &self,
        attempt: &PayoutAttempt,
        audit_event: &AuditEvent,
    ) -> Result<PayoutAttempt, PayoutAttemptStoreError>;

    async fn update_final(
        &self,
        attempt: &PayoutAttempt,
        audit_event: &AuditEvent,
    ) -> Result<PayoutAttempt, PayoutAttemptStoreError>;

    async fn list_for_payrun(
        &self,
        tenant_id: &StandardID<IDTenant>,
        payrun_id: &StandardID<IDPayrun>,
    ) -> Result<Vec<PayoutAttempt>, PayoutAttemptStoreError>;

    async fn get(
        &self,
        tenant_id: &StandardID<IDTenant>,
        id: &StandardID<IDPayoutAttempt>,
    ) -> Result<Option<PayoutAttempt>, PayoutAttemptStoreError>;
}
