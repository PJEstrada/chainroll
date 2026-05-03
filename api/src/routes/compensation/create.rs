use crate::app_state::CompensationState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::compensation::errors::CompensationAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::employee::IDEmployee;
use payroll_service::domain::ids::StandardID;
use payroll_service::services::compensation::create::{CompensationProfileData, CreateRequest};
use payroll_service::services::compensation::service::CompensationService;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct EmployeePath {
    employee_id: String,
}

pub(crate) async fn create_compensation_profile<C: CompensationService>(
    State(state): State<CompensationState<C>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Path(path): Path<EmployeePath>,
    Json(data): Json<CompensationProfileData>,
) -> Result<impl IntoResponse, CompensationAPIError> {
    let response = state
        .compensation_service
        .create(CreateRequest {
            tenant_id,
            employee_id: StandardID::<IDEmployee>::try_from(path.employee_id)?,
            actor_id,
            data,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(response.compensation_profile)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use payroll_service::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfile, CompensationProfileDraft,
    };
    use payroll_service::domain::tenant::IDTenant;
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::compensation::create::CreateResponse;
    use payroll_service::services::compensation::service::MockCompensationService;
    use serde_json::json;

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

    fn valid_body() -> serde_json::Value {
        json!({
            "amount_units": "1000000",
            "token_symbol": "USDC",
            "cadence": "monthly"
        })
    }

    fn build_app(compensation_profile: CompensationProfile) -> Router {
        let mut mock = MockCompensationService::new();
        mock.expect_create().returning(move |_req| {
            Ok(CreateResponse {
                compensation_profile: compensation_profile.clone(),
            })
        });

        Router::new()
            .route(
                "/employees/{employee_id}/compensation-profiles",
                post(create_compensation_profile),
            )
            .with_state(CompensationState::new(mock))
    }

    #[tokio::test]
    async fn missing_actor_returns_400() {
        let server = TestServer::new(build_app(profile())).unwrap();
        let response = server
            .post("/employees/000000000003V/compensation-profiles")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Actor ID is missing" }));
    }

    #[tokio::test]
    async fn create_compensation_profile_returns_201() {
        let server = TestServer::new(build_app(profile())).unwrap();
        let response = server
            .post("/employees/000000000003V/compensation-profiles")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["amount"]["amount_units"], "1000000");
        assert_eq!(body["amount"]["token_symbol"], "USDC");
        assert_eq!(body["cadence"], "monthly");
    }
}
