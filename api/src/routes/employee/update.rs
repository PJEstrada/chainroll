use crate::app_state::EmployeeState;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::employee::create::CreateEmployeeData;
use payroll_service::services::employee::service::EmployeeService;
use payroll_service::services::employee::update::{UpdateEmployeeData, UpdateRequest};

pub(crate) async fn update_employee<E: EmployeeService>(
    State(state): State<EmployeeState<E>>,
    TenantId(tenant_id): TenantId,
    Path(id): Path<String>,
    Json(data): Json<CreateEmployeeData>,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let request = UpdateRequest {
        tenant_id,
        data: UpdateEmployeeData {
            id: StandardID::<IDEmployee>::try_from(id)?,
            identifier: data.identifier,
            first_name: data.first_name,
            last_name: data.last_name,
            divisions: data.divisions,
            culture: data.culture,
            attributes: data.attributes,
            wallet_address: data.wallet_address,
        },
    };
    let response = state.employee_service.update(request).await?;
    Ok((StatusCode::OK, Json(response.employee)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::put;
    use axum_test::TestServer;
    use payroll_service::domain::employee::Employee;
    use payroll_service::services::employee::service::MockEmployeeService;
    use payroll_service::services::employee::update::UpdateResponse;
    use serde_json::json;

    fn build_app(employee: Employee) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_update().returning(move |_req| {
            Ok(UpdateResponse {
                employee: employee.clone(),
            })
        });
        let state = EmployeeState::new(mock);
        Router::new()
            .route("/employees/{id}", put(update_employee))
            .with_state(state)
    }

    fn valid_body() -> serde_json::Value {
        json!({
            "identifier": "EMP-001",
            "first_name": "John",
            "last_name": "Doe",
            "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
        })
    }

    fn test_employee() -> Employee {
        Employee::new("EMP-001".to_string(), "John".to_string(), "Doe".to_string())
            .with_wallet_address(Some(
                payroll_service::domain::wallets::WalletAddress::parse(
                    "0x1234567890abcdef1234567890abcdef12345678",
                )
                .unwrap(),
            ))
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .put("/employees/000000000003V")
            .json(&valid_body())
            .await;
        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn test_missing_required_fields_returns_422() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .put("/employees/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .json(&json!({})) // valid JSON but missing required fields
            .await;
        response.assert_status_unprocessable_entity();
    }

    #[tokio::test]
    async fn test_update_employee_returns_200() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .put("/employees/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status(StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert_eq!(body["identifier"], "EMP-001");
        assert_eq!(body["first_name"], "John");
        assert_eq!(body["last_name"], "Doe");
        assert_eq!(
            body["wallet_address"],
            "0x1234567890AbcdEF1234567890aBcdef12345678"
        );
        assert_eq!(body["metadata"]["status"], "Active");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_invalid_tenant_id_returns_400() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .put("/employees/000000000003V")
            .add_header("x-tenant-id", "not-a-valid-id")
            .json(&valid_body())
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_invalid_employee_id_returns_400() {
        let server = TestServer::new(build_app(test_employee())).unwrap();
        let response = server
            .put("/employees/invalid-id")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;
        response.assert_status_bad_request();
    }
}
