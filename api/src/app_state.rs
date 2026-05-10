use axum::extract::FromRef;
use payroll_service::services::compensation::service::{
    CompensationService, CompensationServiceImpl,
};
use payroll_service::services::datastore::postgres::compensation_store::PgCompensationStore;
use payroll_service::services::datastore::postgres::employee_store::PgEmployeeStore;
use payroll_service::services::datastore::postgres::payout_attempt_store::PgPayoutAttemptStore;
use payroll_service::services::datastore::postgres::payout_instruction_store::PgPayoutInstructionStore;
use payroll_service::services::datastore::postgres::payrun_store::PgPayrunStore;
use payroll_service::services::datastore::postgres::treasury_store::PgTreasuryStore;
use payroll_service::services::employee::service::{EmployeeService, EmployeeServiceImpl};
use payroll_service::services::payout_instruction::service::{
    PayoutInstructionService, PayoutInstructionServiceImpl,
};
use payroll_service::services::payout_submission::service::{
    PayoutSubmissionService, PayoutSubmissionServiceImpl,
};
use payroll_service::services::payrun::service::{PayrunService, PayrunServiceImpl};
use payroll_service::services::stablecoin::tempo_privy::TempoPrivyStablecoinPayoutClient;
use payroll_service::services::treasury::service::{TreasuryService, TreasuryServiceImpl};
use std::sync::Arc;

pub type AppState = AppStateInner<
    EmployeeServiceImpl<PgEmployeeStore>,
    TreasuryServiceImpl<PgTreasuryStore>,
    CompensationServiceImpl<PgCompensationStore>,
    PayrunServiceImpl<PgEmployeeStore, PgCompensationStore, PgTreasuryStore, PgPayrunStore>,
    PayoutInstructionServiceImpl<
        PgEmployeeStore,
        PgTreasuryStore,
        PgPayrunStore,
        PgPayoutInstructionStore,
    >,
    PayoutSubmissionServiceImpl<
        PayoutInstructionServiceImpl<
            PgEmployeeStore,
            PgTreasuryStore,
            PgPayrunStore,
            PgPayoutInstructionStore,
        >,
        PgPayoutAttemptStore,
        TempoPrivyStablecoinPayoutClient,
    >,
>;
#[derive(Debug)]
#[allow(dead_code)]
pub struct AppStateInner<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> {
    pub employee_service: Arc<E>,
    pub treasury_service: Arc<T>,
    pub compensation_service: Arc<C>,
    pub payrun_service: Arc<P>,
    pub payout_instruction_service: Arc<PI>,
    pub payout_submission_service: Arc<PS>,
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> Clone for AppStateInner<E, T, C, P, PI, PS>
{
    fn clone(&self) -> Self {
        Self {
            employee_service: Arc::clone(&self.employee_service),
            treasury_service: Arc::clone(&self.treasury_service),
            compensation_service: Arc::clone(&self.compensation_service),
            payrun_service: Arc::clone(&self.payrun_service),
            payout_instruction_service: Arc::clone(&self.payout_instruction_service),
            payout_submission_service: Arc::clone(&self.payout_submission_service),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> AppStateInner<E, T, C, P, PI, PS>
{
    pub fn new(
        employee_svc: E,
        treasury_svc: T,
        compensation_svc: C,
        payrun_svc: P,
        payout_instruction_svc: PI,
        payout_submission_svc: PS,
    ) -> Self {
        Self {
            employee_service: Arc::new(employee_svc),
            treasury_service: Arc::new(treasury_svc),
            compensation_service: Arc::new(compensation_svc),
            payrun_service: Arc::new(payrun_svc),
            payout_instruction_service: Arc::new(payout_instruction_svc),
            payout_submission_service: Arc::new(payout_submission_svc),
        }
    }
}

#[derive(Debug)]
pub struct EmployeeState<E: EmployeeService> {
    pub employee_service: Arc<E>,
}

impl<E: EmployeeService> Clone for EmployeeState<E> {
    fn clone(&self) -> Self {
        Self {
            employee_service: Arc::clone(&self.employee_service),
        }
    }
}

#[cfg(test)]
impl<E: EmployeeService> EmployeeState<E> {
    pub fn new(employee_svc: E) -> Self {
        Self {
            employee_service: Arc::new(employee_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for EmployeeState<E>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            employee_service: Arc::clone(&state.employee_service),
        }
    }
}

#[derive(Debug)]
pub struct TreasuryState<T: TreasuryService> {
    pub treasury_service: Arc<T>,
}

impl<T: TreasuryService> Clone for TreasuryState<T> {
    fn clone(&self) -> Self {
        Self {
            treasury_service: Arc::clone(&self.treasury_service),
        }
    }
}

#[cfg(test)]
impl<T: TreasuryService> TreasuryState<T> {
    pub fn new(treasury_svc: T) -> Self {
        Self {
            treasury_service: Arc::new(treasury_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for TreasuryState<T>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            treasury_service: Arc::clone(&state.treasury_service),
        }
    }
}

#[derive(Debug)]
pub struct CompensationState<C: CompensationService> {
    pub compensation_service: Arc<C>,
}

impl<C: CompensationService> Clone for CompensationState<C> {
    fn clone(&self) -> Self {
        Self {
            compensation_service: Arc::clone(&self.compensation_service),
        }
    }
}

#[cfg(test)]
impl<C: CompensationService> CompensationState<C> {
    pub fn new(compensation_svc: C) -> Self {
        Self {
            compensation_service: Arc::new(compensation_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for CompensationState<C>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            compensation_service: Arc::clone(&state.compensation_service),
        }
    }
}

#[derive(Debug)]
pub struct PayrunState<P: PayrunService> {
    pub payrun_service: Arc<P>,
}

impl<P: PayrunService> Clone for PayrunState<P> {
    fn clone(&self) -> Self {
        Self {
            payrun_service: Arc::clone(&self.payrun_service),
        }
    }
}

#[cfg(test)]
impl<P: PayrunService> PayrunState<P> {
    pub fn new(payrun_svc: P) -> Self {
        Self {
            payrun_service: Arc::new(payrun_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for PayrunState<P>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            payrun_service: Arc::clone(&state.payrun_service),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PayoutInstructionState<PI: PayoutInstructionService> {
    pub payout_instruction_service: Arc<PI>,
}

impl<PI: PayoutInstructionService> Clone for PayoutInstructionState<PI> {
    fn clone(&self) -> Self {
        Self {
            payout_instruction_service: Arc::clone(&self.payout_instruction_service),
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
impl<PI: PayoutInstructionService> PayoutInstructionState<PI> {
    pub fn new(payout_instruction_svc: PI) -> Self {
        Self {
            payout_instruction_service: Arc::new(payout_instruction_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for PayoutInstructionState<PI>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            payout_instruction_service: Arc::clone(&state.payout_instruction_service),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PayoutSubmissionState<PS: PayoutSubmissionService> {
    pub payout_submission_service: Arc<PS>,
}

impl<PS: PayoutSubmissionService> Clone for PayoutSubmissionState<PS> {
    fn clone(&self) -> Self {
        Self {
            payout_submission_service: Arc::clone(&self.payout_submission_service),
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
impl<PS: PayoutSubmissionService> PayoutSubmissionState<PS> {
    pub fn new(payout_submission_svc: PS) -> Self {
        Self {
            payout_submission_service: Arc::new(payout_submission_svc),
        }
    }
}

impl<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
    PI: PayoutInstructionService,
    PS: PayoutSubmissionService,
> FromRef<AppStateInner<E, T, C, P, PI, PS>> for PayoutSubmissionState<PS>
{
    fn from_ref(state: &AppStateInner<E, T, C, P, PI, PS>) -> Self {
        Self {
            payout_submission_service: Arc::clone(&state.payout_submission_service),
        }
    }
}
