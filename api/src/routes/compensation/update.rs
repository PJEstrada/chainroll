use crate::app_state::CompensationState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::compensation::errors::CompensationAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::compensation::IDCompensationProfile;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::compensation::create::CompensationProfileData;
use payroll_service::services::compensation::service::CompensationService;
use payroll_service::services::compensation::update::UpdateRequest;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CompensationProfilePath {
    employee_id: String,
    id: String,
}

pub(crate) async fn update_compensation_profile<C: CompensationService>(
    State(state): State<CompensationState<C>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Path(path): Path<CompensationProfilePath>,
    Json(data): Json<CompensationProfileData>,
) -> Result<impl IntoResponse, CompensationAPIError> {
    let response = state
        .compensation_service
        .update(UpdateRequest {
            tenant_id,
            employee_id: StandardID::<IDEmployee>::try_from(path.employee_id)?,
            actor_id,
            id: StandardID::<IDCompensationProfile>::try_from(path.id)?,
            data,
        })
        .await?;

    Ok((StatusCode::OK, Json(response.compensation_profile)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::put;
    use axum_test::TestServer;
    use payroll_service::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfile, CompensationProfileDraft,
    };
    use payroll_service::domain::tenant::IDTenant;
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::compensation::service::MockCompensationService;
    use payroll_service::services::compensation::update::UpdateResponse;
    use serde_json::json;

    fn profile(amount_units: u128) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id: StandardID::<IDTenant>::new(),
            employee_id: StandardID::<IDEmployee>::new(),
            amount: CompensationAmount::new(amount_units, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn valid_body() -> serde_json::Value {
        json!({
            "amount_units": "2000000",
            "token_symbol": "USDC",
            "cadence": "monthly"
        })
    }

    fn build_app(compensation_profile: CompensationProfile) -> Router {
        let mut mock = MockCompensationService::new();
        mock.expect_update().returning(move |_req| {
            Ok(UpdateResponse {
                compensation_profile: compensation_profile.clone(),
            })
        });

        Router::new()
            .route(
                "/employees/{employee_id}/compensation-profiles/{id}",
                put(update_compensation_profile),
            )
            .with_state(CompensationState::new(mock))
    }

    #[tokio::test]
    async fn missing_actor_returns_400() {
        let server = TestServer::new(build_app(profile(2_000_000))).unwrap();
        let response = server
            .put("/employees/000000000003V/compensation-profiles/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Actor ID is missing" }));
    }

    #[tokio::test]
    async fn update_compensation_profile_returns_200() {
        let server = TestServer::new(build_app(profile(2_000_000))).unwrap();
        let response = server
            .put("/employees/000000000003V/compensation-profiles/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["amount"]["amount_units"], "2000000");
        assert_eq!(body["amount"]["token_symbol"], "USDC");
    }
}
