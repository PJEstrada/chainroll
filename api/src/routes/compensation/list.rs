use crate::app_state::CompensationState;
use crate::routes::compensation::errors::CompensationAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::compensation::list_for_employee::ListForEmployeeRequest;
use payroll_service::services::compensation::service::CompensationService;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct EmployeePath {
    employee_id: String,
}

pub(crate) async fn list_compensation_profiles<C: CompensationService>(
    State(state): State<CompensationState<C>>,
    TenantId(tenant_id): TenantId,
    Path(path): Path<EmployeePath>,
) -> Result<impl IntoResponse, CompensationAPIError> {
    let response = state
        .compensation_service
        .list_for_employee(ListForEmployeeRequest {
            tenant_id,
            employee_id: StandardID::<IDEmployee>::try_from(path.employee_id)?,
        })
        .await?;

    Ok(Json(response.compensation_profiles).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfile, CompensationProfileDraft,
    };
    use payroll_service::domain::tenant::IDTenant;
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::compensation::list_for_employee::ListForEmployeeResponse;
    use payroll_service::services::compensation::service::MockCompensationService;

    fn profile() -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id: StandardID::<IDTenant>::new(),
            employee_id: StandardID::<IDEmployee>::new(),
            amount: CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn build_app(compensation_profiles: Vec<CompensationProfile>) -> Router {
        let mut mock = MockCompensationService::new();
        mock.expect_list_for_employee().returning(move |_req| {
            Ok(ListForEmployeeResponse {
                compensation_profiles: compensation_profiles.clone(),
            })
        });

        Router::new()
            .route(
                "/employees/{employee_id}/compensation-profiles",
                get(list_compensation_profiles),
            )
            .with_state(CompensationState::new(mock))
    }

    #[tokio::test]
    async fn list_compensation_profiles_returns_profiles() {
        let server = TestServer::new(build_app(vec![profile()])).unwrap();
        let response = server
            .get("/employees/000000000003V/compensation-profiles")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["amount"]["amount_units"], "1000000");
    }
}
