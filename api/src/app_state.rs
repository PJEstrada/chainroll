use payroll_service::services::employee::service::EmployeeServiceImpl;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    employee_service: EmployeeServiceImpl,
}
