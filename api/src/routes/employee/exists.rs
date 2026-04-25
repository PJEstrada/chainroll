use crate::app_state::AppStateInner;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::employee::exists::ExistsRequest;
use payroll_service::services::employee::service::EmployeeService;
use serde_json::json;

pub(crate) async fn employee_exists<E: EmployeeService>(
    State(state): State<AppStateInner<E>>,
    TenantId(tenant_id): TenantId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let response = state
        .employee_service
        .exists(ExistsRequest {
            tenant_id,
            id: StandardID::<IDEmployee>::try_from(id)?,
        })
        .await?;

    Ok(Json(json!({ "exists": response.exists })).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppStateInner;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::services::employee::exists::ExistsResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app(exists: bool) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_exists()
            .returning(move |_req| Ok(ExistsResponse { exists }));
        Router::new()
            .route("/employees/{id}/exists", get(employee_exists))
            .with_state(AppStateInner::new(mock))
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(false)).unwrap();
        let response = server.get("/employees/000000000003V/exists").await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_invalid_employee_id_returns_400() {
        let server = TestServer::new(build_app(false)).unwrap();
        let response = server
            .get("/employees/invalid-id/exists")
            .add_header("x-tenant-id", "000000000003V")
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_employee_exists_returns_200() {
        let server = TestServer::new(build_app(true)).unwrap();
        let response = server
            .get("/employees/000000000003V/exists")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        response.assert_json(&json!({ "exists": true }));
    }
}
