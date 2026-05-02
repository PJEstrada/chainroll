use crate::Result;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{TreasuryAccount, TreasuryAccountQuery};
use crate::services::datastore::TreasuryStore;
use crate::services::treasury::service::{TreasuryServiceImpl, map_store_error};

pub struct ListRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub query: TreasuryAccountQuery,
}

pub struct ListResponse {
    pub treasury_accounts: Vec<TreasuryAccount>,
}

pub(super) async fn execute<S: TreasuryStore>(
    svc: &TreasuryServiceImpl<S>,
    req: ListRequest,
) -> Result<ListResponse> {
    let treasury_accounts = svc
        .store()
        .list(&req.tenant_id, &req.query)
        .await
        .map_err(map_store_error)?;

    Ok(ListResponse { treasury_accounts })
}
