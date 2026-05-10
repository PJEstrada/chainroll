use crate::app_state::PayoutSubmissionState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::payrun::errors::PayrunAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::payrun::IDPayrun;
use payroll_service::services::payout_submission::service::PayoutSubmissionService;
use payroll_service::services::payout_submission::submit::SubmitPayoutsRequest;

pub(crate) async fn submit_payrun_payouts<PS: PayoutSubmissionService>(
    State(state): State<PayoutSubmissionState<PS>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, PayrunAPIError> {
    let response = state
        .payout_submission_service
        .submit(SubmitPayoutsRequest {
            tenant_id,
            actor_id,
            payrun_id: StandardID::<IDPayrun>::try_from(id)?,
        })
        .await?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use payroll_service::domain::ids::StandardID;
    use payroll_service::domain::payrun::IDPayrun;
    use payroll_service::services::payout_submission::service::MockPayoutSubmissionService;
    use payroll_service::services::payout_submission::submit::SubmitPayoutsResponse;

    fn build_app(response: SubmitPayoutsResponse) -> Router {
        let mut mock = MockPayoutSubmissionService::new();
        mock.expect_submit().returning(move |_| {
            Ok(SubmitPayoutsResponse {
                payrun_id: response.payrun_id,
                total_instructions: response.total_instructions,
                submitted: response.submitted,
                failed: response.failed,
                review_required: response.review_required,
                skipped: response.skipped,
                attempts: response.attempts.clone(),
            })
        });

        Router::new()
            .route("/payruns/{id}/submit-payouts", post(submit_payrun_payouts))
            .with_state(PayoutSubmissionState::new(mock))
    }

    fn response() -> SubmitPayoutsResponse {
        SubmitPayoutsResponse {
            payrun_id: StandardID::<IDPayrun>::new(),
            total_instructions: 2,
            submitted: 1,
            failed: 0,
            review_required: 1,
            skipped: 0,
            attempts: Vec::new(),
        }
    }

    #[tokio::test]
    async fn missing_actor_returns_400() {
        let server = TestServer::new(build_app(response())).unwrap();

        let response = server
            .post("/payruns/000000000003V/submit-payouts")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn submit_payouts_returns_summary() {
        let server = TestServer::new(build_app(response())).unwrap();

        let response = server
            .post("/payruns/000000000003V/submit-payouts")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["total_instructions"], 2);
        assert_eq!(body["submitted"], 1);
        assert_eq!(body["review_required"], 1);
    }

    #[tokio::test]
    async fn invalid_payrun_id_returns_400() {
        let server = TestServer::new(build_app(response())).unwrap();

        let response = server
            .post("/payruns/not-an-id/submit-payouts")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .await;

        response.assert_status_bad_request();
    }
}
