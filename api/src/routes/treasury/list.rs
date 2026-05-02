use crate::app_state::TreasuryState;
use crate::routes::tenant_extractor::TenantId;
use crate::routes::treasury::errors::TreasuryAPIError;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use payroll_service::domain::base_metadata::ObjectStatus;
use payroll_service::domain::query::Query as BaseQuery;
use payroll_service::domain::treasury::{TreasuryAccountQuery, TreasuryChain};
use payroll_service::services::treasury::list::ListRequest;
use payroll_service::services::treasury::service::TreasuryService;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize, Default)]
pub(crate) struct ListTreasuryAccountQuery {
    limit: Option<u64>,
    offset: Option<u64>,
    status: Option<String>,
    chain: Option<String>,
    #[serde(default)]
    only_default: bool,
}

pub(crate) async fn list_treasury_accounts<T: TreasuryService>(
    State(state): State<TreasuryState<T>>,
    TenantId(tenant_id): TenantId,
    Query(query): Query<ListTreasuryAccountQuery>,
) -> Result<impl IntoResponse, TreasuryAPIError> {
    let status = query
        .status
        .map(|status| ObjectStatus::from_str(status.as_str()))
        .transpose()
        .map_err(|_| {
            error_stack::Report::new(payroll_service::Error::InvalidInput(
                "invalid treasury account status".to_string(),
            ))
        })?;
    let chain = query
        .chain
        .map(|chain| TreasuryChain::from_str(chain.as_str()))
        .transpose()
        .map_err(|_| {
            error_stack::Report::new(payroll_service::Error::InvalidInput(
                "invalid treasury chain".to_string(),
            ))
        })?;

    let response = state
        .treasury_service
        .list(ListRequest {
            tenant_id,
            query: TreasuryAccountQuery {
                base: BaseQuery {
                    limit: query.limit,
                    offset: query.offset,
                },
                status,
                chain,
                only_default: query.only_default,
            },
        })
        .await?;

    Ok(Json(response.treasury_accounts).into_response())
}
