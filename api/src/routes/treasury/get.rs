use crate::app_state::TreasuryState;
use crate::routes::tenant_extractor::TenantId;
use crate::routes::treasury::errors::TreasuryAPIError;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::treasury::IDTreasuryAccount;
use payroll_service::services::treasury::get::GetRequest;
use payroll_service::services::treasury::service::TreasuryService;

pub(crate) async fn get_treasury_account<T: TreasuryService>(
    State(state): State<TreasuryState<T>>,
    TenantId(tenant_id): TenantId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, TreasuryAPIError> {
    let response = state
        .treasury_service
        .get(GetRequest {
            tenant_id,
            id: StandardID::<IDTreasuryAccount>::try_from(id)?,
        })
        .await?;

    match response.treasury_account {
        Some(treasury_account) => Ok(Json(treasury_account).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}
