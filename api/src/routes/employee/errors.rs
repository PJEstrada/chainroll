use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmployeeAPIError {
    #[error("Tenant ID is missing")]
    TenantIDMissing,
    #[error("invalid id: {0}")]
    InvalidId(#[from] payroll_service::domain::ids::IdError),
    #[error("Internal server error")]
    #[allow(dead_code)]
    InternalError,
    #[error("Service error")]
    ServiceError(error_stack::Report<payroll_service::Error>),
}

impl IntoResponse for EmployeeAPIError {
    fn into_response(self) -> Response {
        let status = match &self {
            EmployeeAPIError::TenantIDMissing => StatusCode::BAD_REQUEST,
            EmployeeAPIError::InvalidId(_) => StatusCode::BAD_REQUEST,
            EmployeeAPIError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            EmployeeAPIError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<error_stack::Report<payroll_service::Error>> for EmployeeAPIError {
    fn from(e: error_stack::Report<payroll_service::Error>) -> Self {
        EmployeeAPIError::ServiceError(e)
    }
}
