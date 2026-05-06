use crate::app_state::PayrunState;
use crate::routes::payrun::errors::PayrunAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use payroll_service::services::payrun::preview::PreviewRequest;
use payroll_service::services::payrun::service::PayrunService;

pub(crate) async fn preview_payrun<P: PayrunService>(
    State(state): State<PayrunState<P>>,
    TenantId(tenant_id): TenantId,
) -> Result<impl IntoResponse, PayrunAPIError> {
    let response = state
        .payrun_service
        .preview(PreviewRequest { tenant_id })
        .await?;

    Ok(Json(response.preview).into_response())
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
    use payroll_service::domain::payrun::{PayrunPreview, PayrunPreviewItem};
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::payrun::preview::PreviewResponse;
    use payroll_service::services::payrun::service::MockPayrunService;
    use serde_json::json;

    fn preview() -> PayrunPreview {
        PayrunPreview::new(
            StandardID::new(),
            vec![PayrunPreviewItem::payable(
                StandardID::<IDEmployee>::new(),
                CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap()).unwrap(),
            )],
        )
    }

    fn build_app(preview: PayrunPreview) -> Router {
        let mut mock = MockPayrunService::new();
        mock.expect_preview().returning(move |_| {
            Ok(PreviewResponse {
                preview: preview.clone(),
            })
        });

        Router::new()
            .route("/payruns/preview", post(preview_payrun))
            .with_state(PayrunState::new(mock))
    }

    #[tokio::test]
    async fn missing_tenant_returns_400() {
        let server = TestServer::new(build_app(preview())).unwrap();

        let response = server.post("/payruns/preview").await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Tenant ID is missing" }));
    }

    #[tokio::test]
    async fn preview_payrun_returns_200() {
        let server = TestServer::new(build_app(preview())).unwrap();

        let response = server
            .post("/payruns/preview")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "ready");
        assert_eq!(body["totals"]["total_employees"], 1);
        assert_eq!(
            body["totals"]["total_amounts"][0]["amount_units"],
            "1000000"
        );
    }
}
