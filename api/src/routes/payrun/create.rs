use crate::app_state::PayrunState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::payrun::errors::PayrunAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::services::payrun::create::{CreatePayrunData, CreateRequest};
use payroll_service::services::payrun::service::PayrunService;

pub(crate) async fn create_payrun<P: PayrunService>(
    State(state): State<PayrunState<P>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Json(data): Json<CreatePayrunData>,
) -> Result<impl IntoResponse, PayrunAPIError> {
    let response = state
        .payrun_service
        .create(CreateRequest {
            tenant_id,
            actor_id,
            data,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(response.payrun)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use payroll_service::domain::compensation::CompensationAmount;
    use payroll_service::domain::employee::IDEmployee;
    use payroll_service::domain::ids::StandardID;
    use payroll_service::domain::payrun::{
        CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem,
    };
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::payrun::create::CreateResponse;
    use payroll_service::services::payrun::service::MockPayrunService;
    use serde_json::json;

    fn payrun() -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::<IDEmployee>::new(),
                    CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                        .unwrap(),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap()
    }

    fn build_app(payrun: Payrun) -> Router {
        let mut mock = MockPayrunService::new();
        mock.expect_create().returning(move |_| {
            Ok(CreateResponse {
                payrun: payrun.clone(),
            })
        });

        Router::new()
            .route("/payruns", post(create_payrun))
            .with_state(PayrunState::new(mock))
    }

    #[tokio::test]
    async fn missing_actor_returns_400() {
        let server = TestServer::new(build_app(payrun())).unwrap();

        let response = server
            .post("/payruns")
            .add_header("x-tenant-id", "000000000003V")
            .json(&json!({ "strict": true }))
            .await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Actor ID is missing" }));
    }

    #[tokio::test]
    async fn create_payrun_returns_201() {
        let server = TestServer::new(build_app(payrun())).unwrap();

        let response = server
            .post("/payruns")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .json(&json!({ "strict": true }))
            .await;

        response.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "created");
        assert_eq!(body["items"][0]["status"], "payable");
        assert_eq!(
            body["totals"]["total_amounts"][0]["amount_units"],
            "1000000"
        );
    }
}
