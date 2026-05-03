use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompensationAPIError {
    #[error("invalid id: {0}")]
    InvalidId(#[from] payroll_service::domain::ids::IdError),
    #[error("Service error")]
    ServiceError(error_stack::Report<payroll_service::Error>),
}

impl IntoResponse for CompensationAPIError {
    fn into_response(self) -> Response {
        let status = match &self {
            CompensationAPIError::InvalidId(_) => StatusCode::BAD_REQUEST,
            CompensationAPIError::ServiceError(report)
                if matches!(report.current_context(), payroll_service::Error::NotFound) =>
            {
                StatusCode::NOT_FOUND
            }
            CompensationAPIError::ServiceError(report)
                if matches!(
                    report.current_context(),
                    payroll_service::Error::InvalidInput(_)
                ) =>
            {
                StatusCode::BAD_REQUEST
            }
            CompensationAPIError::ServiceError(report)
                if matches!(
                    report.current_context(),
                    payroll_service::Error::Conflict(_)
                ) =>
            {
                StatusCode::CONFLICT
            }
            CompensationAPIError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<error_stack::Report<payroll_service::Error>> for CompensationAPIError {
    fn from(e: error_stack::Report<payroll_service::Error>) -> Self {
        CompensationAPIError::ServiceError(e)
    }
}
