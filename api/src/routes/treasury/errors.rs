use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TreasuryAPIError {
    #[error("invalid id: {0}")]
    InvalidId(#[from] payroll_service::domain::ids::IdError),
    #[error("Service error")]
    ServiceError(error_stack::Report<payroll_service::Error>),
}

impl IntoResponse for TreasuryAPIError {
    fn into_response(self) -> Response {
        let status = match &self {
            TreasuryAPIError::InvalidId(_) => StatusCode::BAD_REQUEST,
            TreasuryAPIError::ServiceError(report)
                if matches!(report.current_context(), payroll_service::Error::NotFound) =>
            {
                StatusCode::NOT_FOUND
            }
            TreasuryAPIError::ServiceError(report)
                if matches!(
                    report.current_context(),
                    payroll_service::Error::InvalidInput(_)
                ) =>
            {
                StatusCode::BAD_REQUEST
            }
            TreasuryAPIError::ServiceError(report)
                if matches!(
                    report.current_context(),
                    payroll_service::Error::Conflict(_)
                ) =>
            {
                StatusCode::CONFLICT
            }
            TreasuryAPIError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<error_stack::Report<payroll_service::Error>> for TreasuryAPIError {
    fn from(e: error_stack::Report<payroll_service::Error>) -> Self {
        TreasuryAPIError::ServiceError(e)
    }
}
