use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{IDTreasuryAccount, TreasuryAccount};
use crate::domain::user::IDUser;
use crate::services::datastore::TreasuryStore;
use crate::services::treasury::service::{TreasuryServiceImpl, map_store_error};
use serde_json::json;

pub struct DeactivateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub id: StandardID<IDTreasuryAccount>,
}

pub struct DeactivateResponse {
    pub treasury_account: TreasuryAccount,
}

pub(super) async fn execute<S: TreasuryStore>(
    svc: &TreasuryServiceImpl<S>,
    req: DeactivateRequest,
) -> Result<DeactivateResponse> {
    let mut account = svc
        .store()
        .get(&req.tenant_id, &req.id)
        .await
        .map_err(map_store_error)?
        .ok_or(crate::error::Error::NotFound)?;

    account.deactivate();

    let audit_event = AuditEvent::new(
        *account.tenant_id(),
        req.actor_id,
        AuditEntityType::TreasuryAccount,
        account.id().to_string(),
        AuditEventType::TreasuryAccountDeactivated,
        json!({ "treasury_account": account }),
    );

    let treasury_account = svc
        .store()
        .update(&account, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(DeactivateResponse { treasury_account })
}
