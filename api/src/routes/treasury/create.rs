use crate::app_state::TreasuryState;
use crate::routes::actor_extractor::ActorId;
use crate::routes::tenant_extractor::TenantId;
use crate::routes::treasury::errors::TreasuryAPIError;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use payroll_service::services::treasury::create::{CreateRequest, TreasuryAccountData};
use payroll_service::services::treasury::service::TreasuryService;

pub(crate) async fn create_treasury_account<T: TreasuryService>(
    State(state): State<TreasuryState<T>>,
    TenantId(tenant_id): TenantId,
    ActorId(actor_id): ActorId,
    Json(data): Json<TreasuryAccountData>,
) -> Result<impl IntoResponse, TreasuryAPIError> {
    let response = state
        .treasury_service
        .create(CreateRequest {
            tenant_id,
            actor_id,
            data,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(response.treasury_account)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use payroll_service::domain::ids::StandardID;
    use payroll_service::domain::tenant::IDTenant;
    use payroll_service::domain::treasury::{
        TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use payroll_service::domain::wallets::WalletAddress;
    use payroll_service::services::treasury::create::CreateResponse;
    use payroll_service::services::treasury::service::MockTreasuryService;
    use serde_json::json;

    fn treasury_account() -> TreasuryAccount {
        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id: StandardID::<IDTenant>::new(),
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("pathUSD").unwrap(),
            token_address: WalletAddress::parse("0x20c0000000000000000000000000000000000000")
                .unwrap(),
            token_decimals: 18,
            sender_address: WalletAddress::parse("0x1234567890abcdef1234567890abcdef12345678")
                .unwrap(),
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        })
        .unwrap()
    }

    fn valid_body() -> serde_json::Value {
        json!({
            "name": "Tempo payout source",
            "chain": "tempo-testnet",
            "token_symbol": "pathUSD",
            "token_address": "0x20c0000000000000000000000000000000000000",
            "token_decimals": 18,
            "sender_address": "0x1234567890abcdef1234567890abcdef12345678",
            "custody_provider": "local_key",
            "control_mode": "server_controlled",
            "secret_reference": "env:TEMPO_TREASURY_PRIVATE_KEY",
            "is_default": true
        })
    }

    fn build_app(account: TreasuryAccount) -> Router {
        let mut mock = MockTreasuryService::new();
        mock.expect_create().returning(move |_req| {
            Ok(CreateResponse {
                treasury_account: account.clone(),
            })
        });

        Router::new()
            .route("/treasury-accounts", post(create_treasury_account))
            .with_state(TreasuryState::new(mock))
    }

    #[tokio::test]
    async fn missing_actor_returns_400() {
        let server = TestServer::new(build_app(treasury_account())).unwrap();
        let response = server
            .post("/treasury-accounts")
            .add_header("x-tenant-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status_bad_request();
        response.assert_json(&json!({ "error": "Actor ID is missing" }));
    }

    #[tokio::test]
    async fn create_treasury_account_returns_201() {
        let server = TestServer::new(build_app(treasury_account())).unwrap();
        let response = server
            .post("/treasury-accounts")
            .add_header("x-tenant-id", "000000000003V")
            .add_header("x-actor-id", "000000000003V")
            .json(&valid_body())
            .await;

        response.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "Tempo payout source");
        assert_eq!(body["chain"], "tempo-testnet");
        assert_eq!(body["custody_provider"], "local_key");
        assert_eq!(body["control_mode"], "server_controlled");
        assert_eq!(body["is_default"], true);
    }
}
