use crate::app_state::EmployeeState;
use crate::routes::employee::errors::EmployeeAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use payroll_service::domain::employee::EmployeeQuery;
use payroll_service::domain::query::Query as EmployeeBaseQuery;
use payroll_service::services::employee::list::ListRequest;
use payroll_service::services::employee::service::EmployeeService;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub(crate) struct ListEmployeeQuery {
    limit: Option<u64>,
    offset: Option<u64>,
}

pub(crate) async fn list_employees<E: EmployeeService>(
    State(state): State<EmployeeState<E>>,
    TenantId(tenant_id): TenantId,
    Query(query): Query<ListEmployeeQuery>,
) -> Result<impl IntoResponse, EmployeeAPIError> {
    let response = state
        .employee_service
        .list(ListRequest {
            tenant_id,
            query: EmployeeQuery {
                base: EmployeeBaseQuery {
                    limit: query.limit,
                    offset: query.offset,
                },
                division_id: None,
            },
        })
        .await?;

    Ok(Json(response.employees).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::domain::employee::Employee;
    use payroll_service::services::employee::list::ListResponse;
    use payroll_service::services::employee::service::MockEmployeeService;
    use serde_json::json;

    fn build_app(employees: Vec<Employee>) -> Router {
        let mut mock = MockEmployeeService::new();
        mock.expect_list().returning(move |_req| {
            Ok(ListResponse {
                employees: employees.clone(),
            })
        });
        Router::new()
            .route("/employees", get(list_employees))
            .with_state(EmployeeState::new(mock))
    }

    #[tokio::test]
    async fn test_missing_tenant_returns_400() {
        let server = TestServer::new(build_app(vec![])).unwrap();
        let response = server.get("/employees").await;
        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn test_list_employees_returns_200() {
        let employee = Employee::new("EMP-001".into(), "Jane".into(), "Doe".into());
        let server = TestServer::new(build_app(vec![employee])).unwrap();
        let response = server
            .get("/employees?limit=10&offset=0")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body[0]["identifier"], "EMP-001");
    }
}
