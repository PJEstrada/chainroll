use axum::extract::FromRef;
use payroll_service::services::datastore::postgres::employee_store::PgEmployeeStore;
use payroll_service::services::employee::service::{EmployeeService, EmployeeServiceImpl};
use std::sync::Arc;

pub type AppState = AppStateInner<EmployeeServiceImpl<PgEmployeeStore>>;
#[derive(Debug)]
#[allow(dead_code)]
pub struct AppStateInner<E: EmployeeService> {
    pub employee_service: Arc<E>,
}

impl<E: EmployeeService> Clone for AppStateInner<E> {
    fn clone(&self) -> Self {
        Self {
            employee_service: Arc::clone(&self.employee_service),
        }
    }
}

impl<E: EmployeeService> AppStateInner<E> {
    pub fn new(employee_svc: E) -> Self {
        Self {
            employee_service: Arc::new(employee_svc),
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

impl<E: EmployeeService> FromRef<AppStateInner<E>> for EmployeeState<E> {
    fn from_ref(state: &AppStateInner<E>) -> Self {
        Self {
            employee_service: Arc::clone(&state.employee_service),
        }
    }
}
