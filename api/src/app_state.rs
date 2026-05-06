use axum::extract::FromRef;
use payroll_service::services::compensation::service::{
    CompensationService, CompensationServiceImpl,
};
use payroll_service::services::datastore::postgres::compensation_store::PgCompensationStore;
use payroll_service::services::datastore::postgres::employee_store::PgEmployeeStore;
use payroll_service::services::datastore::postgres::payrun_store::PgPayrunStore;
use payroll_service::services::datastore::postgres::treasury_store::PgTreasuryStore;
use payroll_service::services::employee::service::{EmployeeService, EmployeeServiceImpl};
use payroll_service::services::payrun::service::{PayrunService, PayrunServiceImpl};
use payroll_service::services::treasury::service::{TreasuryService, TreasuryServiceImpl};
use std::sync::Arc;

pub type AppState = AppStateInner<
    EmployeeServiceImpl<PgEmployeeStore>,
    TreasuryServiceImpl<PgTreasuryStore>,
    CompensationServiceImpl<PgCompensationStore>,
    PayrunServiceImpl<PgEmployeeStore, PgCompensationStore, PgTreasuryStore, PgPayrunStore>,
>;
#[derive(Debug)]
#[allow(dead_code)]
pub struct AppStateInner<
    E: EmployeeService,
    T: TreasuryService,
    C: CompensationService,
    P: PayrunService,
> {
    pub employee_service: Arc<E>,
    pub treasury_service: Arc<T>,
    pub compensation_service: Arc<C>,
    pub payrun_service: Arc<P>,
}

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService> Clone
    for AppStateInner<E, T, C, P>
{
    fn clone(&self) -> Self {
        Self {
            employee_service: Arc::clone(&self.employee_service),
            treasury_service: Arc::clone(&self.treasury_service),
            compensation_service: Arc::clone(&self.compensation_service),
            payrun_service: Arc::clone(&self.payrun_service),
        }
    }
}

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService>
    AppStateInner<E, T, C, P>
{
    pub fn new(employee_svc: E, treasury_svc: T, compensation_svc: C, payrun_svc: P) -> Self {
        Self {
            employee_service: Arc::new(employee_svc),
            treasury_service: Arc::new(treasury_svc),
            compensation_service: Arc::new(compensation_svc),
            payrun_service: Arc::new(payrun_svc),
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

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService>
    FromRef<AppStateInner<E, T, C, P>> for EmployeeState<E>
{
    fn from_ref(state: &AppStateInner<E, T, C, P>) -> Self {
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

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService>
    FromRef<AppStateInner<E, T, C, P>> for TreasuryState<T>
{
    fn from_ref(state: &AppStateInner<E, T, C, P>) -> Self {
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

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService>
    FromRef<AppStateInner<E, T, C, P>> for CompensationState<C>
{
    fn from_ref(state: &AppStateInner<E, T, C, P>) -> Self {
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

impl<E: EmployeeService, T: TreasuryService, C: CompensationService, P: PayrunService>
    FromRef<AppStateInner<E, T, C, P>> for PayrunState<P>
{
    fn from_ref(state: &AppStateInner<E, T, C, P>) -> Self {
        Self {
            payrun_service: Arc::clone(&state.payrun_service),
        }
    }
}
