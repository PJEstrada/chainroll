use crate::app_state::AppStateInner;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::services::employee::create::{CreateEmployeeData, CreateRequest};
use payroll_service::services::employee::service::EmployeeService;

pub(crate) async fn create_employee<E: EmployeeService>(
    State(state): State<AppStateInner<E>>,
    TenantId(tenant_id): TenantId,
    Json(data): Json<CreateEmployeeData>,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let request = CreateRequest { tenant_id, data };
    let response = state.employee_service.create(request).await?;
    Ok((StatusCode::CREATED, Json(response.employee)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppStateInner;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use payroll_service::domain::employee::Employee;
    use payroll_service::services::employee::create::CreateResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app(employee: Employee) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_create().returning(move |_req| {
            Ok(CreateResponse {
                employee: employee.clone(),
            })
        });
        let state = AppStateInner::new(mock);
        Router::new()
            .route("/employees", post(create_employee))
            .with_state(state)
    }

    fn valid_body() -> serde_json::Value {
        json!({
            "identifier": "EMP-001",
            "first_name": "John",
            "last_name": "Doe"
        })
    }

    fn test_employee() -> Employee {
        Employee::new("EMP-001".to_string(), "John".to_string(), "Doe".to_string())
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server.post("/employees").json(&valid_body()).await;
        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn test_missing_required_fields_returns_422() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .post("/employees")
            .add_header("x-tenant-id", "000000000003V")
            .json(&json!({})) // valid JSON but missing required fields
            .await;
        response.assert_status_unprocessable_entity();
    }

    #[tokio::test]
    async fn test_create_employee_returns_201() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .post("/employees")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["identifier"], "EMP-001");
        assert_eq!(body["first_name"], "John");
        assert_eq!(body["last_name"], "Doe");
        assert_eq!(body["metadata"]["status"], "Active");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_invalid_tenant_id_returns_400() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .post("/employees")
            .add_header("x-tenant-id", "not-a-valid-id")
            .json(&valid_body())
            .await;
        response.assert_status_bad_request();
    }
}
