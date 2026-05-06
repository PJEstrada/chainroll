use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayrunAPIError {
    #[error("Service error")]
    ServiceError(error_stack::Report<payroll_service::Error>),
}

impl IntoResponse for PayrunAPIError {
    fn into_response(self) -> Response {
        let status = match &self {
            PayrunAPIError::ServiceError(report)
                if matches!(
                    report.current_context(),
                    payroll_service::Error::InvalidInput(_)
                ) =>
            {
                StatusCode::BAD_REQUEST
            }
            PayrunAPIError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<error_stack::Report<payroll_service::Error>> for PayrunAPIError {
    fn from(e: error_stack::Report<payroll_service::Error>) -> Self {
        PayrunAPIError::ServiceError(e)
    }
}
