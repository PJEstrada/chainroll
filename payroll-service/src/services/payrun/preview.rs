use crate::Result;
use crate::domain::compensation::CompensationProfile;
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::payrun::{
    PayrunPreview, PayrunPreviewBlocker, PayrunPreviewError, PayrunPreviewItem,
};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TreasuryAccount;
use crate::error::Error;
use crate::services::datastore::{CompensationStore, EmployeeStore, PayrunStore, TreasuryStore};
use crate::services::payrun::service::PayrunServiceImpl;
use error_stack::ResultExt;
use std::collections::HashMap;

pub struct PreviewRequest {
    pub tenant_id: StandardID<IDTenant>,
}

pub struct PreviewResponse {
    pub preview: PayrunPreview,
}

pub struct PreviewCalculationInput {
    pub tenant_id: StandardID<IDTenant>,
    pub employees: Vec<Employee>,
    pub compensation_profiles: Vec<CompensationProfile>,
    pub default_treasury_accounts: Vec<TreasuryAccount>,
}

pub(super) async fn execute<
    E: EmployeeStore,
    C: CompensationStore,
    T: TreasuryStore,
    P: PayrunStore,
>(
    svc: &PayrunServiceImpl<E, C, T, P>,
    req: PreviewRequest,
) -> Result<PreviewResponse> {
    let employees = svc
        .employee_store()
        .list_active(&req.tenant_id)
        .await
        .change_context(Error::Database)?;
    let compensation_profiles = svc
        .compensation_store()
        .list_active_for_tenant(&req.tenant_id)
        .await
        .change_context(Error::Database)?;
    let default_treasury_accounts = svc
        .treasury_store()
        .list_default_active(&req.tenant_id)
        .await
        .change_context(Error::Database)?;

    let preview = calculate_preview(PreviewCalculationInput {
        tenant_id: req.tenant_id,
        employees,
        compensation_profiles,
        default_treasury_accounts,
    })
    .change_context(Error::InvalidInput("invalid payrun preview".to_string()))?;

    Ok(PreviewResponse { preview })
}

pub fn calculate_preview(
    input: PreviewCalculationInput,
) -> std::result::Result<PayrunPreview, PayrunPreviewError> {
    let compensation_by_employee = input
        .compensation_profiles
        .iter()
        .map(|profile| (*profile.employee_id(), profile))
        .collect::<HashMap<StandardID<IDEmployee>, &CompensationProfile>>();
    let treasury_by_token = input
        .default_treasury_accounts
        .iter()
        .map(|account| (account.token_symbol().as_str(), account))
        .collect::<HashMap<&str, &TreasuryAccount>>();

    let mut items = Vec::with_capacity(input.employees.len());

    for employee in &input.employees {
        let mut blockers = Vec::new();

        if employee.wallet_address().is_none() {
            blockers.push(PayrunPreviewBlocker::MissingWallet);
        }

        let amount = match compensation_by_employee.get(employee.id()) {
            Some(profile) => {
                let amount = profile.amount().clone();
                match treasury_by_token.get(amount.token_symbol().as_str()) {
                    Some(treasury_account) if treasury_account.requires_user_signature() => {
                        blockers.push(PayrunPreviewBlocker::TreasuryRequiresUserSignature);
                    }
                    Some(_) => {}
                    None => blockers.push(PayrunPreviewBlocker::MissingTreasuryAccount),
                }
                Some(amount)
            }
            None => {
                blockers.push(PayrunPreviewBlocker::MissingActiveCompensationProfile);
                None
            }
        };

        let item = if blockers.is_empty() {
            PayrunPreviewItem::payable(
                *employee.id(),
                amount.ok_or(PayrunPreviewError::ItemRequiresAmountOrBlocker)?,
            )
        } else {
            PayrunPreviewItem::blocked(*employee.id(), amount, blockers)?
        };
        items.push(item);
    }

    Ok(PayrunPreview::new(input.tenant_id, items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfileDraft,
    };
    use crate::domain::payrun::PayrunPreviewStatus;
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::wallets::WalletAddress;
    use crate::services::datastore::{
        MockCompensationStore, MockEmployeeStore, MockPayrunStore, MockTreasuryStore,
    };

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn employee(identifier: &str, wallet_address: Option<WalletAddress>) -> Employee {
        Employee::new(
            identifier.to_string(),
            "Jane".to_string(),
            "Doe".to_string(),
        )
        .with_wallet_address(wallet_address)
    }

    fn compensation_profile(
        tenant_id: StandardID<IDTenant>,
        employee_id: StandardID<IDEmployee>,
        token_symbol: &str,
    ) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id,
            employee_id,
            amount: CompensationAmount::new(1_000_000, TokenSymbol::parse(token_symbol).unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn treasury_account(
        tenant_id: StandardID<IDTenant>,
        token_symbol: &str,
        control_mode: TreasuryControlMode,
    ) -> TreasuryAccount {
        let (custody_provider, provider_wallet_id, secret_reference) = match control_mode {
            TreasuryControlMode::ServerControlled => (
                TreasuryCustodyProvider::LocalKey,
                None,
                Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            ),
            TreasuryControlMode::UserSignatureRequired => (
                TreasuryCustodyProvider::Privy,
                Some("privy-wallet-1".to_string()),
                None,
            ),
            TreasuryControlMode::UserDelegated | TreasuryControlMode::ExternalExecution => {
                unreachable!("test helper only supports preview-relevant modes")
            }
        };

        TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id,
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse(token_symbol).unwrap(),
            token_address: wallet("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: wallet("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider,
            control_mode,
            provider_wallet_id,
            provider_owner_id: None,
            secret_reference,
            is_default: true,
        })
        .unwrap()
    }

    #[test]
    fn calculates_ready_preview() {
        let tenant_id = StandardID::new();
        let employee = employee(
            "EMP-001",
            Some(wallet("0x1234567890abcdef1234567890abcdef12345678")),
        );
        let profile = compensation_profile(tenant_id, *employee.id(), "USDC");

        let preview = calculate_preview(PreviewCalculationInput {
            tenant_id,
            employees: vec![employee],
            compensation_profiles: vec![profile],
            default_treasury_accounts: vec![treasury_account(
                tenant_id,
                "USDC",
                TreasuryControlMode::ServerControlled,
            )],
        })
        .unwrap();

        assert_eq!(preview.status(), PayrunPreviewStatus::Ready);
        assert!(preview.items()[0].is_payable());
        assert_eq!(
            preview.totals().unwrap().total_amounts()[0].amount_units(),
            1_000_000
        );
    }

    #[test]
    fn blocks_employee_without_wallet() {
        let tenant_id = StandardID::new();
        let employee = employee("EMP-001", None);
        let profile = compensation_profile(tenant_id, *employee.id(), "USDC");

        let preview = calculate_preview(PreviewCalculationInput {
            tenant_id,
            employees: vec![employee],
            compensation_profiles: vec![profile],
            default_treasury_accounts: vec![treasury_account(
                tenant_id,
                "USDC",
                TreasuryControlMode::ServerControlled,
            )],
        })
        .unwrap();

        assert_eq!(preview.status(), PayrunPreviewStatus::Blocked);
        assert_eq!(
            preview.items()[0].blockers(),
            &[PayrunPreviewBlocker::MissingWallet]
        );
    }

    #[test]
    fn blocks_employee_without_active_compensation_profile() {
        let tenant_id = StandardID::new();
        let employee = employee(
            "EMP-001",
            Some(wallet("0x1234567890abcdef1234567890abcdef12345678")),
        );

        let preview = calculate_preview(PreviewCalculationInput {
            tenant_id,
            employees: vec![employee],
            compensation_profiles: Vec::new(),
            default_treasury_accounts: vec![treasury_account(
                tenant_id,
                "USDC",
                TreasuryControlMode::ServerControlled,
            )],
        })
        .unwrap();

        assert_eq!(
            preview.items()[0].blockers(),
            &[PayrunPreviewBlocker::MissingActiveCompensationProfile]
        );
    }

    #[test]
    fn blocks_employee_without_matching_default_treasury_account() {
        let tenant_id = StandardID::new();
        let employee = employee(
            "EMP-001",
            Some(wallet("0x1234567890abcdef1234567890abcdef12345678")),
        );
        let profile = compensation_profile(tenant_id, *employee.id(), "USDC");

        let preview = calculate_preview(PreviewCalculationInput {
            tenant_id,
            employees: vec![employee],
            compensation_profiles: vec![profile],
            default_treasury_accounts: vec![treasury_account(
                tenant_id,
                "pathUSD",
                TreasuryControlMode::ServerControlled,
            )],
        })
        .unwrap();

        assert_eq!(
            preview.items()[0].blockers(),
            &[PayrunPreviewBlocker::MissingTreasuryAccount]
        );
    }

    #[test]
    fn blocks_employee_when_treasury_requires_user_signature() {
        let tenant_id = StandardID::new();
        let employee = employee(
            "EMP-001",
            Some(wallet("0x1234567890abcdef1234567890abcdef12345678")),
        );
        let profile = compensation_profile(tenant_id, *employee.id(), "USDC");

        let preview = calculate_preview(PreviewCalculationInput {
            tenant_id,
            employees: vec![employee],
            compensation_profiles: vec![profile],
            default_treasury_accounts: vec![treasury_account(
                tenant_id,
                "USDC",
                TreasuryControlMode::UserSignatureRequired,
            )],
        })
        .unwrap();

        assert_eq!(
            preview.items()[0].blockers(),
            &[PayrunPreviewBlocker::TreasuryRequiresUserSignature]
        );
    }

    #[tokio::test]
    async fn execute_loads_preview_inputs_from_stores() {
        let tenant_id = StandardID::new();
        let employee = employee(
            "EMP-001",
            Some(wallet("0x1234567890abcdef1234567890abcdef12345678")),
        );
        let profile = compensation_profile(tenant_id, *employee.id(), "USDC");
        let treasury = treasury_account(tenant_id, "USDC", TreasuryControlMode::ServerControlled);

        let mut employee_store = MockEmployeeStore::new();
        employee_store
            .expect_list_active()
            .withf(move |actual_tenant_id| actual_tenant_id == &tenant_id)
            .returning(move |_| Ok(vec![employee.clone()]));

        let mut compensation_store = MockCompensationStore::new();
        compensation_store
            .expect_list_active_for_tenant()
            .withf(move |actual_tenant_id| actual_tenant_id == &tenant_id)
            .returning(move |_| Ok(vec![profile.clone()]));

        let mut treasury_store = MockTreasuryStore::new();
        treasury_store
            .expect_list_default_active()
            .withf(move |actual_tenant_id| actual_tenant_id == &tenant_id)
            .returning(move |_| Ok(vec![treasury.clone()]));

        let svc = PayrunServiceImpl::new(
            employee_store,
            compensation_store,
            treasury_store,
            MockPayrunStore::new(),
        );

        let response = execute(&svc, PreviewRequest { tenant_id }).await.unwrap();

        assert_eq!(response.preview.status(), PayrunPreviewStatus::Ready);
    }
}
