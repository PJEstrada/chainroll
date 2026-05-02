use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{IDTreasuryAccount, TreasuryAccount};
use crate::domain::user::IDUser;
use crate::services::datastore::TreasuryStore;
use crate::services::treasury::create::{TreasuryAccountData, draft_from_data};
use crate::services::treasury::service::{TreasuryServiceImpl, map_store_error};
use error_stack::ResultExt;
use serde_json::json;

pub struct UpdateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub id: StandardID<IDTreasuryAccount>,
    pub data: TreasuryAccountData,
}

pub struct UpdateResponse {
    pub treasury_account: TreasuryAccount,
}

pub(super) async fn execute<S: TreasuryStore>(
    svc: &TreasuryServiceImpl<S>,
    req: UpdateRequest,
) -> Result<UpdateResponse> {
    let existing = svc
        .store()
        .get(&req.tenant_id, &req.id)
        .await
        .map_err(map_store_error)?
        .ok_or(crate::error::Error::NotFound)?;

    let account = TreasuryAccount::restore(
        req.id,
        existing.metadata().clone(),
        draft_from_data(req.tenant_id, req.data)?,
    )
    .change_context(crate::error::Error::InvalidInput(
        "invalid treasury account".to_string(),
    ))?;

    let audit_event = AuditEvent::new(
        *account.tenant_id(),
        req.actor_id,
        AuditEntityType::TreasuryAccount,
        account.id().to_string(),
        AuditEventType::TreasuryAccountUpdated,
        json!({ "treasury_account": account }),
    );

    let treasury_account = svc
        .store()
        .update(&account, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(UpdateResponse { treasury_account })
}
