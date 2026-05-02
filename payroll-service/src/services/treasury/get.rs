use crate::Result;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{IDTreasuryAccount, TreasuryAccount};
use crate::services::datastore::TreasuryStore;
use crate::services::treasury::service::{TreasuryServiceImpl, map_store_error};

pub struct GetRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub id: StandardID<IDTreasuryAccount>,
}

pub struct GetResponse {
    pub treasury_account: Option<TreasuryAccount>,
}

pub(super) async fn execute<S: TreasuryStore>(
    svc: &TreasuryServiceImpl<S>,
    req: GetRequest,
) -> Result<GetResponse> {
    let treasury_account = svc
        .store()
        .get(&req.tenant_id, &req.id)
        .await
        .map_err(map_store_error)?;

    Ok(GetResponse { treasury_account })
}
