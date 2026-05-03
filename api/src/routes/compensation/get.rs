use crate::app_state::CompensationState;
use crate::routes::compensation::errors::CompensationAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::compensation::IDCompensationProfile;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::compensation::get::GetRequest;
use payroll_service::services::compensation::service::CompensationService;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CompensationProfilePath {
    employee_id: String,
    id: String,
}

pub(crate) async fn get_compensation_profile<C: CompensationService>(
    State(state): State<CompensationState<C>>,
    TenantId(tenant_id): TenantId,
    Path(path): Path<CompensationProfilePath>,
) -> Result<impl IntoResponse, CompensationAPIError> {
    let response = state
        .compensation_service
        .get(GetRequest {
            tenant_id,
            employee_id: StandardID::<IDEmployee>::try_from(path.employee_id)?,
            id: StandardID::<IDCompensationProfile>::try_from(path.id)?,
        })
        .await?;

    match response.compensation_profile {
        Some(compensation_profile) => Ok(Json(compensation_profile).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
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
    use payroll_service::services::compensation::get::GetResponse;
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

    fn build_app(compensation_profile: Option<CompensationProfile>) -> Router {
        let mut mock = MockCompensationService::new();
        mock.expect_get().returning(move |_req| {
            Ok(GetResponse {
                compensation_profile: compensation_profile.clone(),
            })
        });

        Router::new()
            .route(
                "/employees/{employee_id}/compensation-profiles/{id}",
                get(get_compensation_profile),
            )
            .with_state(CompensationState::new(mock))
    }

    #[tokio::test]
    async fn get_compensation_profile_returns_200() {
        let server = TestServer::new(build_app(Some(profile()))).unwrap();
        let response = server
            .get("/employees/000000000003V/compensation-profiles/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["amount"]["amount_units"], "1000000");
    }

    #[tokio::test]
    async fn get_compensation_profile_returns_404() {
        let server = TestServer::new(build_app(None)).unwrap();
        let response = server
            .get("/employees/000000000003V/compensation-profiles/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_not_found();
    }
}
