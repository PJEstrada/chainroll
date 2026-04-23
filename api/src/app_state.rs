use std::sync::Arc;

use payroll_service::services::datastore::postgres::employee_store::PgEmployeeStore;
use payroll_service::services::employee::service::{EmployeeService, EmployeeServiceImpl};

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
