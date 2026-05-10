use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::ids::StandardID;
use crate::domain::payout_attempt::{PayoutAttempt, PayoutAttemptStatus};
use crate::domain::payout_instruction::{IDPayoutInstruction, PayoutInstruction};
use crate::domain::payrun::IDPayrun;
use crate::domain::tenant::IDTenant;
use crate::domain::user::IDUser;
use crate::error::Error;
use crate::services::datastore::PayoutAttemptStore;
use crate::services::payout_instruction::generate::GenerateRequest;
use crate::services::payout_instruction::service::PayoutInstructionService;
use crate::services::payout_submission::service::{
    PayoutSubmissionServiceImpl, map_attempt_store_error,
};
use crate::services::stablecoin::client::{
    ProviderPayoutReference, StablecoinPayoutClient, StablecoinPayoutClientError,
    StablecoinPayoutOutcome, StablecoinPayoutRequest,
};
use error_stack::ResultExt;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

pub struct SubmitPayoutsRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub actor_id: StandardID<IDUser>,
    pub payrun_id: StandardID<IDPayrun>,
}

#[derive(Debug, Serialize)]
pub struct SubmitPayoutsResponse {
    pub payrun_id: StandardID<IDPayrun>,
    pub total_instructions: usize,
    pub submitted: usize,
    pub failed: usize,
    pub review_required: usize,
    pub skipped: usize,
    pub attempts: Vec<PayoutAttempt>,
}

#[derive(Default)]
struct SubmissionCounts {
    submitted: usize,
    failed: usize,
    review_required: usize,
    skipped: usize,
}

pub(super) async fn execute<PI, A, C>(
    svc: &PayoutSubmissionServiceImpl<PI, A, C>,
    req: SubmitPayoutsRequest,
) -> Result<SubmitPayoutsResponse>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    let instruction_response = svc
        .payout_instruction_service()
        .generate(GenerateRequest {
            tenant_id: req.tenant_id,
            actor_id: req.actor_id,
            payrun_id: req.payrun_id,
        })
        .await?;
    let instructions = instruction_response.payout_instructions;
    let existing_attempts = svc
        .payout_attempt_store()
        .list_for_payrun(&req.tenant_id, &req.payrun_id)
        .await
        .map_err(map_attempt_store_error)?;
    let attempts_by_instruction = attempts_by_instruction(existing_attempts);

    let mut counts = SubmissionCounts::default();
    let mut response_attempts = Vec::new();

    for instruction in &instructions {
        let previous_attempts = attempts_by_instruction
            .get(instruction.id())
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        if should_skip(previous_attempts, &mut counts, &mut response_attempts) {
            continue;
        }

        let started = start_attempt(svc, instruction, req.actor_id, previous_attempts).await?;
        let outcome = svc
            .stablecoin_client()
            .submit_payout(StablecoinPayoutRequest::from_instruction(instruction))
            .await;
        let finalized = finalize_attempt(&started, outcome)?;
        let finalized = persist_final_attempt(svc, &finalized, req.actor_id).await?;
        update_counts(finalized.status(), &mut counts);
        response_attempts.push(finalized);
    }

    Ok(SubmitPayoutsResponse {
        payrun_id: req.payrun_id,
        total_instructions: instructions.len(),
        submitted: counts.submitted,
        failed: counts.failed,
        review_required: counts.review_required,
        skipped: counts.skipped,
        attempts: response_attempts,
    })
}

async fn start_attempt<PI, A, C>(
    svc: &PayoutSubmissionServiceImpl<PI, A, C>,
    instruction: &PayoutInstruction,
    actor_id: StandardID<IDUser>,
    previous_attempts: &[PayoutAttempt],
) -> Result<PayoutAttempt>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    let attempt_number = next_attempt_number(previous_attempts);
    let attempt = PayoutAttempt::started(
        *instruction.tenant_id(),
        *instruction.payrun_id(),
        *instruction.id(),
        attempt_number,
    )
    .change_context(Error::InvalidInput("invalid payout attempt".to_string()))?;
    let audit_event = audit_event(
        &attempt,
        actor_id,
        AuditEventType::PayoutAttemptStarted,
        json!({
            "payout_attempt": attempt,
            "payout_instruction_id": instruction.id(),
        }),
    );

    svc.payout_attempt_store()
        .create_started(&attempt, &audit_event)
        .await
        .map_err(map_attempt_store_error)
}

async fn persist_final_attempt<PI, A, C>(
    svc: &PayoutSubmissionServiceImpl<PI, A, C>,
    attempt: &PayoutAttempt,
    actor_id: StandardID<IDUser>,
) -> Result<PayoutAttempt>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    let event_type = match attempt.status() {
        PayoutAttemptStatus::Submitted => AuditEventType::PayoutAttemptSubmitted,
        PayoutAttemptStatus::Failed => AuditEventType::PayoutAttemptFailed,
        PayoutAttemptStatus::ReviewRequired => AuditEventType::PayoutAttemptReviewRequired,
        PayoutAttemptStatus::Started => {
            return Err(Error::InvalidInput("attempt was not finalized".to_string()).into());
        }
    };
    let audit_event = audit_event(
        attempt,
        actor_id,
        event_type,
        json!({ "payout_attempt": attempt }),
    );

    svc.payout_attempt_store()
        .update_final(attempt, &audit_event)
        .await
        .map_err(map_attempt_store_error)
}

fn finalize_attempt(
    started: &PayoutAttempt,
    outcome: std::result::Result<StablecoinPayoutOutcome, StablecoinPayoutClientError>,
) -> Result<PayoutAttempt> {
    let attempt = match outcome {
        Ok(StablecoinPayoutOutcome::Submitted(submitted)) => started.mark_submitted(
            submitted.provider_reference().as_str().to_string(),
            submitted
                .transaction_hash()
                .map(|hash| hash.as_str().to_string()),
        ),
        Ok(StablecoinPayoutOutcome::Rejected(rejected)) => started.mark_failed(
            format!("stablecoin payout rejected: {:?}", rejected.reason()),
            provider_reference_to_string(rejected.provider_reference()),
        ),
        Ok(StablecoinPayoutOutcome::ReviewRequired(review)) => started.mark_review_required(
            format!("stablecoin payout requires review: {:?}", review.reason()),
            provider_reference_to_string(review.provider_reference()),
        ),
        Err(err) => started.mark_failed(err.to_string(), None),
    }
    .change_context(Error::InvalidInput(
        "invalid payout attempt transition".to_string(),
    ))?;

    Ok(attempt)
}

fn should_skip(
    previous_attempts: &[PayoutAttempt],
    counts: &mut SubmissionCounts,
    response_attempts: &mut Vec<PayoutAttempt>,
) -> bool {
    if let Some(attempt) = attempt_with_status(previous_attempts, PayoutAttemptStatus::Submitted) {
        counts.skipped += 1;
        response_attempts.push(attempt.clone());
        return true;
    }

    if let Some(attempt) =
        attempt_with_status(previous_attempts, PayoutAttemptStatus::ReviewRequired)
    {
        counts.review_required += 1;
        response_attempts.push(attempt.clone());
        return true;
    }

    if let Some(attempt) = attempt_with_status(previous_attempts, PayoutAttemptStatus::Started) {
        counts.skipped += 1;
        response_attempts.push(attempt.clone());
        return true;
    }

    false
}

fn update_counts(status: PayoutAttemptStatus, counts: &mut SubmissionCounts) {
    match status {
        PayoutAttemptStatus::Submitted => counts.submitted += 1,
        PayoutAttemptStatus::Failed => counts.failed += 1,
        PayoutAttemptStatus::ReviewRequired => counts.review_required += 1,
        PayoutAttemptStatus::Started => counts.skipped += 1,
    }
}

fn attempts_by_instruction(
    attempts: Vec<PayoutAttempt>,
) -> HashMap<StandardID<IDPayoutInstruction>, Vec<PayoutAttempt>> {
    let mut attempts_by_instruction = HashMap::<StandardID<IDPayoutInstruction>, Vec<_>>::new();

    for attempt in attempts {
        attempts_by_instruction
            .entry(*attempt.payout_instruction_id())
            .or_default()
            .push(attempt);
    }

    attempts_by_instruction
}

fn attempt_with_status(
    attempts: &[PayoutAttempt],
    status: PayoutAttemptStatus,
) -> Option<&PayoutAttempt> {
    attempts
        .iter()
        .filter(|attempt| attempt.status() == status)
        .max_by_key(|attempt| attempt.attempt_number())
}

fn next_attempt_number(previous_attempts: &[PayoutAttempt]) -> u32 {
    previous_attempts
        .iter()
        .map(PayoutAttempt::attempt_number)
        .max()
        .unwrap_or(0)
        + 1
}

fn provider_reference_to_string(
    provider_reference: Option<&ProviderPayoutReference>,
) -> Option<String> {
    provider_reference.map(|reference| reference.as_str().to_string())
}

fn audit_event(
    attempt: &PayoutAttempt,
    actor_id: StandardID<IDUser>,
    event_type: AuditEventType,
    payload: serde_json::Value,
) -> AuditEvent {
    AuditEvent::new(
        *attempt.tenant_id(),
        actor_id,
        AuditEntityType::PayoutAttempt,
        attempt.id().to_string(),
        event_type,
        payload,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::Employee;
    use crate::domain::payrun::{CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::wallets::WalletAddress;
    use crate::services::datastore::MockPayoutAttemptStore;
    use crate::services::payout_instruction::generate::GenerateResponse;
    use crate::services::payout_instruction::service::MockPayoutInstructionService;
    use crate::services::stablecoin::client::{
        ProviderPayoutReference, ReviewRequiredStablecoinPayout, StablecoinPayoutReviewReason,
        SubmittedStablecoinPayout, TransactionHash,
    };
    use crate::services::stablecoin::fake::FakeStablecoinPayoutClient;

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn instruction() -> PayoutInstruction {
        let payrun = Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                        .unwrap(),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap();
        let item = &payrun.items()[0];
        let employee = Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(*item.employee_id())
            .with_wallet_address(Some(wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")));
        let treasury = TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id: *payrun.tenant_id(),
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("USDC").unwrap(),
            token_address: wallet("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: wallet("0x1234567890abcdef1234567890abcdef12345678"),
            custody_provider: TreasuryCustodyProvider::Privy,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: Some("privy-wallet-id".to_string()),
            provider_owner_id: None,
            secret_reference: None,
            is_default: true,
        })
        .unwrap();

        PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap()
    }

    fn submitted_outcome() -> StablecoinPayoutOutcome {
        let hash = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        StablecoinPayoutOutcome::Submitted(SubmittedStablecoinPayout::new(
            ProviderPayoutReference::parse(hash).unwrap(),
            Some(TransactionHash::parse(hash).unwrap()),
        ))
    }

    fn review_required_outcome() -> StablecoinPayoutOutcome {
        StablecoinPayoutOutcome::ReviewRequired(ReviewRequiredStablecoinPayout::new(
            StablecoinPayoutReviewReason::AmbiguousProviderResponse,
            None,
        ))
    }

    fn request_for(instruction: &PayoutInstruction) -> SubmitPayoutsRequest {
        SubmitPayoutsRequest {
            tenant_id: *instruction.tenant_id(),
            actor_id: StandardID::new(),
            payrun_id: *instruction.payrun_id(),
        }
    }

    fn instruction_service(instruction: PayoutInstruction) -> MockPayoutInstructionService {
        let mut service = MockPayoutInstructionService::new();
        service.expect_generate().returning(move |_| {
            Ok(GenerateResponse {
                payout_instructions: vec![instruction.clone()],
            })
        });
        service
    }

    fn attempt_store(existing_attempts: Vec<PayoutAttempt>) -> MockPayoutAttemptStore {
        let mut store = MockPayoutAttemptStore::new();
        store
            .expect_list_for_payrun()
            .returning(move |_, _| Ok(existing_attempts.clone()));
        store
            .expect_create_started()
            .returning(|attempt, _| Ok(attempt.clone()));
        store
            .expect_update_final()
            .returning(|attempt, _| Ok(attempt.clone()));
        store
    }

    #[tokio::test]
    async fn submits_instruction_and_records_provider_request() {
        let instruction = instruction();
        let client = FakeStablecoinPayoutClient::with_success(submitted_outcome());
        let svc = PayoutSubmissionServiceImpl::new(
            instruction_service(instruction.clone()),
            attempt_store(Vec::new()),
            client.clone(),
        );

        let response = execute(&svc, request_for(&instruction)).await.unwrap();

        assert_eq!(response.total_instructions, 1);
        assert_eq!(response.submitted, 1);
        assert_eq!(response.failed, 0);
        assert_eq!(client.requests().len(), 1);
        assert_eq!(client.requests()[0].instruction_id(), instruction.id());
    }

    #[tokio::test]
    async fn skips_already_submitted_instruction() {
        let instruction = instruction();
        let attempt = PayoutAttempt::started(
            *instruction.tenant_id(),
            *instruction.payrun_id(),
            *instruction.id(),
            1,
        )
        .unwrap()
        .mark_submitted(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Some("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
        )
        .unwrap();
        let mut store = MockPayoutAttemptStore::new();
        store
            .expect_list_for_payrun()
            .returning(move |_, _| Ok(vec![attempt.clone()]));
        store.expect_create_started().never();
        store.expect_update_final().never();
        let client = FakeStablecoinPayoutClient::default();
        let svc = PayoutSubmissionServiceImpl::new(
            instruction_service(instruction.clone()),
            store,
            client.clone(),
        );

        let response = execute(&svc, request_for(&instruction)).await.unwrap();

        assert_eq!(response.skipped, 1);
        assert_eq!(response.submitted, 0);
        assert!(client.requests().is_empty());
    }

    #[tokio::test]
    async fn failed_instruction_is_retried_with_next_attempt_number() {
        let instruction = instruction();
        let failed = PayoutAttempt::started(
            *instruction.tenant_id(),
            *instruction.payrun_id(),
            *instruction.id(),
            1,
        )
        .unwrap()
        .mark_failed("provider rejected", None)
        .unwrap();
        let client = FakeStablecoinPayoutClient::with_success(submitted_outcome());
        let svc = PayoutSubmissionServiceImpl::new(
            instruction_service(instruction.clone()),
            attempt_store(vec![failed]),
            client,
        );

        let response = execute(&svc, request_for(&instruction)).await.unwrap();

        assert_eq!(response.submitted, 1);
        assert_eq!(response.attempts[0].attempt_number(), 2);
    }

    #[tokio::test]
    async fn review_required_instruction_is_not_retried() {
        let instruction = instruction();
        let review_required = PayoutAttempt::started(
            *instruction.tenant_id(),
            *instruction.payrun_id(),
            *instruction.id(),
            1,
        )
        .unwrap()
        .mark_review_required("timeout after submission", None)
        .unwrap();
        let mut store = MockPayoutAttemptStore::new();
        store
            .expect_list_for_payrun()
            .returning(move |_, _| Ok(vec![review_required.clone()]));
        store.expect_create_started().never();
        store.expect_update_final().never();
        let client = FakeStablecoinPayoutClient::default();
        let svc = PayoutSubmissionServiceImpl::new(
            instruction_service(instruction.clone()),
            store,
            client.clone(),
        );

        let response = execute(&svc, request_for(&instruction)).await.unwrap();

        assert_eq!(response.review_required, 1);
        assert!(client.requests().is_empty());
    }

    #[tokio::test]
    async fn ambiguous_outcome_marks_attempt_review_required() {
        let instruction = instruction();
        let client = FakeStablecoinPayoutClient::with_success(review_required_outcome());
        let svc = PayoutSubmissionServiceImpl::new(
            instruction_service(instruction.clone()),
            attempt_store(Vec::new()),
            client,
        );

        let response = execute(&svc, request_for(&instruction)).await.unwrap();

        assert_eq!(response.review_required, 1);
        assert_eq!(
            response.attempts[0].status(),
            PayoutAttemptStatus::ReviewRequired
        );
    }
}
