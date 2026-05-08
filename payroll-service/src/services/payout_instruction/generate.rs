use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::employee::{Employee, IDEmployee};
use crate::domain::ids::StandardID;
use crate::domain::payout_instruction::PayoutInstruction;
use crate::domain::payrun::{IDPayrun, IDPayrunItem, Payrun, PayrunItem, PayrunItemStatus};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TreasuryAccount;
use crate::domain::user::IDUser;
use crate::error::Error;
use crate::services::datastore::{
    EmployeeStore, PayoutInstructionStore, PayrunStore, TreasuryStore,
};
use crate::services::payout_instruction::service::{
    PayoutInstructionServiceImpl, map_instruction_store_error, map_payrun_store_error,
};
use error_stack::ResultExt;
use serde_json::json;
use std::collections::{HashMap, HashSet};

pub struct GenerateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub payrun_id: StandardID<IDPayrun>,
}

#[derive(Debug)]
pub struct GenerateResponse {
    pub payout_instructions: Vec<PayoutInstruction>,
}

pub(super) async fn execute<
    E: EmployeeStore,
    T: TreasuryStore,
    P: PayrunStore,
    I: PayoutInstructionStore,
>(
    svc: &PayoutInstructionServiceImpl<E, T, P, I>,
    req: GenerateRequest,
) -> Result<GenerateResponse> {
    let payrun = svc
        .payrun_store()
        .get(&req.tenant_id, &req.payrun_id)
        .await
        .map_err(map_payrun_store_error)?
        .ok_or(Error::NotFound)?;

    let existing = svc
        .payout_instruction_store()
        .list_for_payrun(&req.tenant_id, &req.payrun_id)
        .await
        .map_err(map_instruction_store_error)?;

    let payable_items = payable_items(&payrun);
    if has_instructions_for_all_payable_items(&existing, &payable_items) {
        return Ok(GenerateResponse {
            payout_instructions: existing,
        });
    }

    let existing_item_ids = existing
        .iter()
        .map(|instruction| *instruction.payrun_item_id())
        .collect::<HashSet<_>>();
    let treasury_accounts = svc
        .treasury_store()
        .list_default_active(&req.tenant_id)
        .await
        .change_context(Error::Database)?;
    let treasury_by_token = treasury_accounts
        .iter()
        .map(|account| (account.token_symbol().as_str(), account))
        .collect::<HashMap<_, _>>();

    let mut instructions = Vec::new();
    for item in payable_items {
        if existing_item_ids.contains(item.id()) {
            continue;
        }

        let employee = load_employee(svc.employee_store(), &req.tenant_id, item.employee_id())
            .await?
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "payable employee {} was not found",
                    item.employee_id()
                ))
            })?;
        let amount = item.amount().ok_or_else(|| {
            Error::InvalidInput(format!("payable item {} is missing an amount", item.id()))
        })?;
        let treasury_account = treasury_by_token
            .get(amount.token_symbol().as_str())
            .copied()
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "missing default treasury account for token {}",
                    amount.token_symbol()
                ))
            })?;

        let instruction = build_instruction(&payrun, item, &employee, treasury_account)?;
        instructions.push(instruction);
    }

    let audit_events = instructions
        .iter()
        .map(|instruction| {
            AuditEvent::new(
                *instruction.tenant_id(),
                req.actor_id,
                AuditEntityType::PayoutInstruction,
                instruction.id().to_string(),
                AuditEventType::PayoutInstructionCreated,
                json!({ "payout_instruction": instruction }),
            )
        })
        .collect::<Vec<_>>();

    let payout_instructions = svc
        .payout_instruction_store()
        .create_many_idempotent(&instructions, &audit_events)
        .await
        .map_err(map_instruction_store_error)?;

    Ok(GenerateResponse {
        payout_instructions,
    })
}

fn payable_items(payrun: &Payrun) -> Vec<&PayrunItem> {
    payrun
        .items()
        .iter()
        .filter(|item| item.status() == PayrunItemStatus::Payable)
        .collect()
}

fn has_instructions_for_all_payable_items(
    existing: &[PayoutInstruction],
    payable_items: &[&PayrunItem],
) -> bool {
    let existing_item_ids = existing
        .iter()
        .map(|instruction| *instruction.payrun_item_id())
        .collect::<HashSet<StandardID<IDPayrunItem>>>();

    payable_items
        .iter()
        .all(|item| existing_item_ids.contains(item.id()))
}

async fn load_employee<E: EmployeeStore>(
    store: &E,
    tenant_id: &StandardID<IDTenant>,
    employee_id: &StandardID<IDEmployee>,
) -> Result<Option<Employee>> {
    store
        .get(tenant_id, employee_id)
        .await
        .change_context(Error::Database)
}

fn build_instruction(
    payrun: &Payrun,
    item: &PayrunItem,
    employee: &Employee,
    treasury_account: &TreasuryAccount,
) -> Result<PayoutInstruction> {
    PayoutInstruction::new(payrun, item, employee, treasury_account).change_context(
        Error::InvalidInput("invalid payout instruction".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::Employee;
    use crate::domain::payrun::{CreatePayrunOptions, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::wallets::WalletAddress;
    use crate::services::datastore::{
        MockEmployeeStore, MockPayoutInstructionStore, MockPayrunStore, MockTreasuryStore,
    };

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn amount(token_symbol: &str) -> CompensationAmount {
        CompensationAmount::new(1_000_000, TokenSymbol::parse(token_symbol).unwrap()).unwrap()
    }

    fn payrun() -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    amount("USDC"),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap()
    }

    fn employee(id: StandardID<IDEmployee>) -> Employee {
        Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(id)
            .with_wallet_address(Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")))
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

    #[tokio::test]
    async fn generates_missing_instructions_for_payable_items() {
        let tenant_id = StandardID::new();
        let mut payrun = payrun();
        payrun = Payrun::restore(
            *payrun.id(),
            tenant_id,
            payrun.status(),
            payrun.items().to_vec(),
            payrun.created_at(),
            payrun.updated_at(),
        )
        .unwrap();
        let item = payrun.items()[0].clone();
        let employee = employee(*item.employee_id());
        let treasury = treasury_account(tenant_id);

        let mut payrun_store = MockPayrunStore::new();
        payrun_store
            .expect_get()
            .returning(move |_, _| Ok(Some(payrun.clone())));

        let mut instruction_store = MockPayoutInstructionStore::new();
        instruction_store
            .expect_list_for_payrun()
            .returning(|_, _| Ok(Vec::new()));
        instruction_store
            .expect_create_many_idempotent()
            .withf(|instructions, audit_events| {
                instructions.len() == 1
                    && audit_events.len() == 1
                    && audit_events[0].entity_type() == AuditEntityType::PayoutInstruction
                    && audit_events[0].event_type() == AuditEventType::PayoutInstructionCreated
            })
            .returning(|instructions, _| Ok(instructions.to_vec()));

        let mut employee_store = MockEmployeeStore::new();
        employee_store
            .expect_get()
            .returning(move |_, _| Ok(Some(employee.clone())));

        let mut treasury_store = MockTreasuryStore::new();
        treasury_store
            .expect_list_default_active()
            .returning(move |_| Ok(vec![treasury.clone()]));

        let svc = PayoutInstructionServiceImpl::new(
            employee_store,
            treasury_store,
            payrun_store,
            instruction_store,
        );
        let response = execute(
            &svc,
            GenerateRequest {
                tenant_id,
                actor_id: StandardID::new(),
                payrun_id: StandardID::new(),
            },
        )
        .await
        .unwrap();

        assert_eq!(response.payout_instructions.len(), 1);
        assert_eq!(response.payout_instructions[0].payrun_item_id(), item.id());
    }

    #[tokio::test]
    async fn returns_existing_instructions_without_rebuilding() {
        let tenant_id = StandardID::new();
        let mut payrun = payrun();
        payrun = Payrun::restore(
            *payrun.id(),
            tenant_id,
            payrun.status(),
            payrun.items().to_vec(),
            payrun.created_at(),
            payrun.updated_at(),
        )
        .unwrap();
        let item = &payrun.items()[0];
        let instruction = PayoutInstruction::new(
            &payrun,
            item,
            &employee(*item.employee_id()),
            &treasury_account(tenant_id),
        )
        .unwrap();

        let mut payrun_store = MockPayrunStore::new();
        payrun_store
            .expect_get()
            .returning(move |_, _| Ok(Some(payrun.clone())));

        let mut instruction_store = MockPayoutInstructionStore::new();
        instruction_store
            .expect_list_for_payrun()
            .returning(move |_, _| Ok(vec![instruction.clone()]));

        let svc = PayoutInstructionServiceImpl::new(
            MockEmployeeStore::new(),
            MockTreasuryStore::new(),
            payrun_store,
            instruction_store,
        );
        let response = execute(
            &svc,
            GenerateRequest {
                tenant_id,
                actor_id: StandardID::new(),
                payrun_id: StandardID::new(),
            },
        )
        .await
        .unwrap();

        assert_eq!(response.payout_instructions.len(), 1);
    }

    #[tokio::test]
    async fn returns_not_found_when_payrun_is_missing() {
        let mut payrun_store = MockPayrunStore::new();
        payrun_store.expect_get().returning(|_, _| Ok(None));

        let svc = PayoutInstructionServiceImpl::new(
            MockEmployeeStore::new(),
            MockTreasuryStore::new(),
            payrun_store,
            MockPayoutInstructionStore::new(),
        );

        let result = execute(
            &svc,
            GenerateRequest {
                tenant_id: StandardID::new(),
                actor_id: StandardID::new(),
                payrun_id: StandardID::new(),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(result.current_context(), Error::NotFound));
    }
}
