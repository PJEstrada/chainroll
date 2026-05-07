use crate::app_state::PayrunState;
use crate::routes::payrun::errors::PayrunAPIError;
use crate::routes::tenant_extractor::TenantId;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::payrun::IDPayrun;
use payroll_service::services::payrun::get::GetRequest;
use payroll_service::services::payrun::service::PayrunService;

pub(crate) async fn get_payrun<P: PayrunService>(
    State(state): State<PayrunState<P>>,
    TenantId(tenant_id): TenantId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, PayrunAPIError> {
    let response = state
        .payrun_service
        .get(GetRequest {
            tenant_id,
            id: StandardID::<IDPayrun>::try_from(id)?,
        })
        .await?;

    match response.payrun {
        Some(payrun) => Ok(Json(payrun).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use payroll_service::domain::compensation::CompensationAmount;
    use payroll_service::domain::employee::IDEmployee;
    use payroll_service::domain::payrun::{
        CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem,
    };
    use payroll_service::domain::treasury::TokenSymbol;
    use payroll_service::services::payrun::get::GetResponse;
    use payroll_service::services::payrun::service::MockPayrunService;

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

    fn build_app(payrun: Option<Payrun>) -> Router {
        let mut mock = MockPayrunService::new();
        mock.expect_get().returning(move |_| {
            Ok(GetResponse {
                payrun: payrun.clone(),
            })
        });

        Router::new()
            .route("/payruns/{id}", get(get_payrun))
            .with_state(PayrunState::new(mock))
    }

    #[tokio::test]
    async fn get_payrun_returns_200() {
        let server = TestServer::new(build_app(Some(payrun()))).unwrap();

        let response = server
            .get("/payruns/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "created");
        assert_eq!(body["items"][0]["status"], "payable");
    }

    #[tokio::test]
    async fn get_payrun_returns_404() {
        let server = TestServer::new(build_app(None)).unwrap();

        let response = server
            .get("/payruns/000000000003V")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn invalid_payrun_id_returns_400() {
        let server = TestServer::new(build_app(Some(payrun()))).unwrap();

        let response = server
            .get("/payruns/not-an-id")
            .add_header("x-tenant-id", "000000000003V")
            .await;

        response.assert_status_bad_request();
    }
}
