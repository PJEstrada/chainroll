use crate::app_state::TreasuryState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::tenant_extractor::TenantId;
use crate::routes::treasury::errors::TreasuryAPIError;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::treasury::IDTreasuryAccount;
use payroll_service::services::treasury::create::TreasuryAccountData;
use payroll_service::services::treasury::service::TreasuryService;
use payroll_service::services::treasury::update::UpdateRequest;

pub(crate) async fn update_treasury_account<T: TreasuryService>(
    State(state): State<TreasuryState<T>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Path(id): Path<String>,
    Json(data): Json<TreasuryAccountData>,
) -> Result<impl IntoResponse, TreasuryAPIError> {
    let response = state
        .treasury_service
        .update(UpdateRequest {
            tenant_id,
            actor_id,
            id: StandardID::<IDTreasuryAccount>::try_from(id)?,
            data,
        })
        .await?;

    Ok((StatusCode::OK, Json(response.treasury_account)).into_response())
}
