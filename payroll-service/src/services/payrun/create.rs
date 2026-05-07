use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::ids::StandardID;
use crate::domain::payrun::{CreatePayrunOptions, Payrun};
use crate::domain::tenant::IDTenant;
use crate::domain::user::IDUser;
use crate::error::Error;
use crate::services::datastore::{CompensationStore, EmployeeStore, PayrunStore, TreasuryStore};
use crate::services::payrun::preview::{PreviewRequest, execute as preview_execute};
use crate::services::payrun::service::{PayrunServiceImpl, map_store_error};
use error_stack::ResultExt;
use serde::Deserialize;
use serde_json::json;

fn default_strict() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayrunData {
    #[serde(default = "default_strict")]
    pub strict: bool,
    #[serde(default)]
    pub exclude_unpayable: bool,
}

pub struct CreateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub data: CreatePayrunData,
}

#[derive(Debug)]
pub struct CreateResponse {
    pub payrun: Payrun,
}

pub(super) async fn execute<
    E: EmployeeStore,
    C: CompensationStore,
    T: TreasuryStore,
    P: PayrunStore,
>(
    svc: &PayrunServiceImpl<E, C, T, P>,
    req: CreateRequest,
) -> Result<CreateResponse> {
    let preview = preview_execute(
        svc,
        PreviewRequest {
            tenant_id: req.tenant_id,
        },
    )
    .await?
    .preview;

    let payrun = Payrun::new(
        preview,
        CreatePayrunOptions::new(req.data.strict, req.data.exclude_unpayable),
    )
    .change_context(Error::InvalidInput("invalid payrun".to_string()))?;

    let audit_event = AuditEvent::new(
        *payrun.tenant_id(),
        req.actor_id,
        AuditEntityType::Payrun,
        payrun.id().to_string(),
        AuditEventType::PayrunCreated,
        json!({ "payrun": payrun }),
    );

    let payrun = svc
        .payrun_store()
        .create(&payrun, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(CreateResponse { payrun })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfile, CompensationProfileDraft,
    };
    use crate::domain::employee::Employee;
    use crate::domain::payrun::PayrunItemStatus;
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::wallets::WalletAddress;
    use crate::services::datastore::{
        MockCompensationStore, MockEmployeeStore, MockPayrunStore, MockTreasuryStore,
    };

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn employee(with_wallet: bool) -> Employee {
        let wallet_address =
            with_wallet.then(|| wallet("0x1234567890abcdef1234567890abcdef12345678"));
        Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_wallet_address(wallet_address)
    }

    fn compensation_profile(
        tenant_id: StandardID<IDTenant>,
        employee_id: crate::domain::ids::StandardID<crate::domain::employee::IDEmployee>,
    ) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id,
            employee_id,
            amount: CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn treasury_account(tenant_id: StandardID<IDTenant>) -> TreasuryAccount {
        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id,
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("USDC").unwrap(),
            token_address: wallet("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: wallet("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        })
        .unwrap()
    }

    fn service(
        employee: Employee,
        profile: CompensationProfile,
        treasury: TreasuryAccount,
    ) -> PayrunServiceImpl<
        MockEmployeeStore,
        MockCompensationStore,
        MockTreasuryStore,
        MockPayrunStore,
    > {
        let mut employee_store = MockEmployeeStore::new();
        employee_store
            .expect_list_active()
            .returning(move |_| Ok(vec![employee.clone()]));

        let mut compensation_store = MockCompensationStore::new();
        compensation_store
            .expect_list_active_for_tenant()
            .returning(move |_| Ok(vec![profile.clone()]));

        let mut treasury_store = MockTreasuryStore::new();
        treasury_store
            .expect_list_default_active()
            .returning(move |_| Ok(vec![treasury.clone()]));

        let mut payrun_store = MockPayrunStore::new();
        payrun_store
            .expect_create()
            .withf(|payrun, audit_event| {
                audit_event.entity_type() == AuditEntityType::Payrun
                    && audit_event.event_type() == AuditEventType::PayrunCreated
                    && audit_event.entity_id() == payrun.id().to_string()
            })
            .returning(|payrun, _| Ok(payrun.clone()));

        PayrunServiceImpl::new(
            employee_store,
            compensation_store,
            treasury_store,
            payrun_store,
        )
    }

    #[tokio::test]
    async fn creates_strict_payrun_from_ready_preview() {
        let tenant_id = StandardID::new();
        let employee = employee(true);
        let profile = compensation_profile(tenant_id, *employee.id());
        let treasury = treasury_account(tenant_id);
        let svc = service(employee, profile, treasury);

        let response = execute(
            &svc,
            CreateRequest {
                tenant_id,
                actor_id: StandardID::new(),
                data: CreatePayrunData {
                    strict: true,
                    exclude_unpayable: false,
                },
            },
        )
        .await
        .unwrap();

        assert_eq!(response.payrun.items().len(), 1);
        assert_eq!(
            response.payrun.items()[0].status(),
            PayrunItemStatus::Payable
        );
    }

    #[tokio::test]
    async fn rejects_payrun_when_exclusion_leaves_no_payable_items() {
        let tenant_id = StandardID::new();
        let employee = employee(false);
        let profile = compensation_profile(tenant_id, *employee.id());
        let treasury = treasury_account(tenant_id);
        let svc = service(employee, profile, treasury);

        let result = execute(
            &svc,
            CreateRequest {
                tenant_id,
                actor_id: StandardID::new(),
                data: CreatePayrunData {
                    strict: false,
                    exclude_unpayable: true,
                },
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(result.current_context(), Error::InvalidInput(_)));
    }
}
