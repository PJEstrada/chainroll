use crate::app_state::EmployeeState;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::employee::delete::DeleteRequest;
use payroll_service::services::employee::service::EmployeeService;

pub(crate) async fn delete_employee<E: EmployeeService>(
    State(state): State<EmployeeState<E>>,
    TenantId(tenant_id): TenantId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    state
        .employee_service
        .delete(DeleteRequest {
            tenant_id,
            id: StandardID::<IDEmployee>::try_from(id)?,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::delete;
    use axum_test::TestServer;
    use payroll_service::services::employee::delete::DeleteResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app() -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_delete().returning(|_req| Ok(DeleteResponse));
        Router::new()
            .route("/employees/{id}", delete(delete_employee))
            .with_state(EmployeeState::new(mock))
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app()).unwrap();
        let response = server.delete("/employees/000000000003V").await;
        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn test_invalid_employee_id_returns_400() {
        let server = TestServer::new(build_app()).unwrap();
        let response = server
            .delete("/employees/invalid-id")
            .add_header("x-tenant-id", "000000000003V")
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_delete_employee_returns_204() {
        let server = TestServer::new(build_app()).unwrap();
        let response = server
            .delete("/employees/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status(StatusCode::NO_CONTENT);
    }
}
