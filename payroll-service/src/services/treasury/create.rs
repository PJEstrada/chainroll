use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::{
    TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
    TreasuryCustodyProvider,
};
use crate::domain::user::IDUser;
use crate::domain::wallets::WalletAddress;
use crate::services::datastore::TreasuryStore;
use crate::services::treasury::service::{TreasuryServiceImpl, map_store_error};
use error_stack::ResultExt;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
pub struct TreasuryAccountData {
    pub name: String,
    pub chain: TreasuryChain,
    pub token_symbol: String,
    pub token_address: String,
    pub token_decimals: u8,
    pub sender_address: String,
    pub custody_provider: TreasuryCustodyProvider,
    pub control_mode: TreasuryControlMode,
    pub provider_wallet_id: Option<String>,
    pub provider_owner_id: Option<String>,
    pub secret_reference: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

pub struct CreateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub data: TreasuryAccountData,
}

#[derive(Debug)]
pub struct CreateResponse {
    pub treasury_account: TreasuryAccount,
}

pub(super) async fn execute<S: TreasuryStore>(
    svc: &TreasuryServiceImpl<S>,
    req: CreateRequest,
) -> Result<CreateResponse> {
    let account = TreasuryAccount::new(draft_from_data(req.tenant_id, req.data)?).change_context(
        crate::error::Error::InvalidInput("invalid treasury account".to_string()),
    )?;

    let audit_event = AuditEvent::new(
        *account.tenant_id(),
        req.actor_id,
        AuditEntityType::TreasuryAccount,
        account.id().to_string(),
        AuditEventType::TreasuryAccountCreated,
        json!({ "treasury_account": account }),
    );

    let treasury_account = svc
        .store()
        .create(&account, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(CreateResponse { treasury_account })
}

pub(super) fn draft_from_data(
    tenant_id: StandardID<IDTenant>,
    data: TreasuryAccountData,
) -> Result<TreasuryAccountDraft> {
    let token_symbol = TokenSymbol::parse(data.token_symbol).change_context(
        crate::error::Error::InvalidInput("invalid token symbol".to_string()),
    )?;
    let token_address = WalletAddress::parse(data.token_address).change_context(
        crate::error::Error::InvalidInput("invalid token address".to_string()),
    )?;
    let sender_address = WalletAddress::parse(data.sender_address).change_context(
        crate::error::Error::InvalidInput("invalid sender address".to_string()),
    )?;

    Ok(TreasuryAccountDraft {
        tenant_id,
        name: data.name,
        chain: data.chain,
        token_symbol,
        token_address,
        token_decimals: data.token_decimals,
        sender_address,
        custody_provider: data.custody_provider,
        control_mode: data.control_mode,
        provider_wallet_id: data.provider_wallet_id,
        provider_owner_id: data.provider_owner_id,
        secret_reference: data.secret_reference,
        is_default: data.is_default,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockTreasuryStore;

    fn request() -> CreateRequest {
        CreateRequest {
            tenant_id: StandardID::new(),
            actor_id: StandardID::new(),
            data: TreasuryAccountData {
                name: "Tempo payout source".to_string(),
                chain: TreasuryChain::TempoTestnet,
                token_symbol: "pathUSD".to_string(),
                token_address: "0x20c0000000000000000000000000000000000000".to_string(),
                token_decimals: 18,
                sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                custody_provider: TreasuryCustodyProvider::LocalKey,
                control_mode: TreasuryControlMode::ServerControlled,
                provider_wallet_id: None,
                provider_owner_id: None,
                secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
                is_default: true,
            },
        }
    }

    #[tokio::test]
    async fn creates_treasury_account_with_audit_event() {
        let mut store = MockTreasuryStore::new();
        store
            .expect_create()
            .withf(|account, audit_event| {
                audit_event.entity_type() == AuditEntityType::TreasuryAccount
                    && audit_event.event_type() == AuditEventType::TreasuryAccountCreated
                    && audit_event.entity_id() == account.id().to_string()
            })
            .returning(|account, _| Ok(account.clone()));

        let svc = TreasuryServiceImpl::new(store);
        let response = execute(&svc, request()).await.unwrap();

        assert_eq!(response.treasury_account.name(), "Tempo payout source");
        assert!(response.treasury_account.is_default());
    }

    #[tokio::test]
    async fn returns_invalid_input_for_bad_token_address() {
        let store = MockTreasuryStore::new();
        let svc = TreasuryServiceImpl::new(store);
        let mut req = request();
        req.data.token_address = "not-an-address".to_string();

        let result = execute(&svc, req).await;

        assert!(matches!(
            result.unwrap_err().current_context(),
            crate::error::Error::InvalidInput(_)
        ));
    }
}
