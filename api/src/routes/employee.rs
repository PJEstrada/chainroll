use crate::app_state::AppStateInner;
use crate::routes::errors::EmployeeAPIError;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::{HeaderMap, StatusCode};
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::tenant::IDTenant;
use payroll_service::services::employee::get::GetRequest;
use payroll_service::services::employee::service::EmployeeService;

pub(crate) async fn get_employee<E: EmployeeService>(
    State(state): State<AppStateInner<E>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let tenant_id = match headers.get("x-tenant-id") {
        Some(tenant_id) => {
            let raw = tenant_id
                .to_str()
                .map_err(|_| EmployeeAPIError::TenantIDMissing)?;
            StandardID::<IDTenant>::try_from(raw.to_string())?
        }
        None => return Err(EmployeeAPIError::TenantIDMissing),
    };
    let employee_id = StandardID::<IDEmployee>::try_from(id.clone())?;
    // call state.employee_service.get(...)
    let request = GetRequest {
        tenant_id,
        id: employee_id,
    };
    let response = state.employee_service.get(request).await?;
    match response.employee {
        Some(employee) => Ok(Json(employee).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppStateInner;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::domain::employee::Employee;
    use payroll_service::services::employee::get::GetResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app(e: Option<Employee>) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_get().returning(move |_req| {
            Ok(GetResponse {
                employee: e.clone(),
            })
        });
        let state = AppStateInner::new(mock);
        Router::new()
            .route("/employees/{id}", get(get_employee))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(None)).unwrap();
        let response = server
            .get("/employees/000000000003V")
            // no header!
            .await;
        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn test_valid_request_returns_not_found() {
        let server = TestServer::new(build_app(None)).unwrap();
        let response = server
            .get("/employees/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_get_employee_check_employee_id() {
        let server = TestServer::new(build_app(None)).unwrap();

        let response = server
            .get("/employees/invalid-id")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "invalid id: invalid id format" }));
    }

    #[tokio::test]
    async fn test_employee_200() {
        let test_employee =
            Employee::new("EMP-001".to_string(), "John".to_string(), "Doe".to_string());
        let server = TestServer::new(build_app(Some(test_employee))).unwrap();

        let response = server
            .get("/employees/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["identifier"], "EMP-001");
        assert_eq!(body["first_name"], "John");
        assert_eq!(body["last_name"], "Doe");
        assert_eq!(body["divisions"], json!([]));
        assert_eq!(body["culture"], json!(null));
        assert_eq!(body["attributes"], json!(null));
        assert_eq!(body["metadata"]["status"], "Active");
        assert!(body["id"].is_string(), "id should be present");
        assert!(
            body["metadata"]["created"].is_string(),
            "created should be present"
        );
        assert!(
            body["metadata"]["updated"].is_string(),
            "updated should be present"
        );
    }
}
