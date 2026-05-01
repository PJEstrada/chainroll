use crate::app_state::AppStateInner;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use payroll_service::services::employee::count::CountRequest;
use payroll_service::services::employee::service::EmployeeService;
use serde_json::json;

pub(crate) async fn count_employees<E: EmployeeService>(
    State(state): State<AppStateInner<E>>,
    TenantId(tenant_id): TenantId,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let response = state
        .employee_service
        .count(CountRequest { tenant_id })
        .await?;

    Ok(Json(json!({ "count": response.count })).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppStateInner;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::services::employee::count::CountResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app(count: i64) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_count()
            .returning(move |_req| Ok(CountResponse { count }));
        Router::new()
            .route("/employees/count", get(count_employees))
            .with_state(AppStateInner::new(mock))
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(0)).unwrap();
        let response = server.get("/employees/count").await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_count_employees_returns_200() {
        let server = TestServer::new(build_app(3)).unwrap();
        let response = server
            .get("/employees/count")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        response.assert_json(&json!({ "count": 3 }));
    }
}
