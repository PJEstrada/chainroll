use crate::services::stablecoin::client::{
    StablecoinPayoutClient, StablecoinPayoutClientError, StablecoinPayoutOutcome,
    StablecoinPayoutRequest,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

type FakeResult = Result<StablecoinPayoutOutcome, StablecoinPayoutClientError>;

#[derive(Debug, Clone, Default)]
pub struct FakeStablecoinPayoutClient {
    requests: Arc<Mutex<Vec<StablecoinPayoutRequest>>>,
    responses: Arc<Mutex<VecDeque<FakeResult>>>,
}

impl FakeStablecoinPayoutClient {
    pub fn new(responses: impl IntoIterator<Item = FakeResult>) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
        }
    }

    pub fn with_success(response: StablecoinPayoutOutcome) -> Self {
        Self::new([Ok(response)])
    }

    pub fn requests(&self) -> Vec<StablecoinPayoutRequest> {
        self.requests
            .lock()
            .expect("fake stablecoin payout requests mutex is poisoned")
            .clone()
    }
}

impl StablecoinPayoutClient for FakeStablecoinPayoutClient {
    async fn submit_payout(
        &self,
        request: StablecoinPayoutRequest,
    ) -> Result<StablecoinPayoutOutcome, StablecoinPayoutClientError> {
        self.requests
            .lock()
            .expect("fake stablecoin payout requests mutex is poisoned")
            .push(request);

        self.responses
            .lock()
            .expect("fake stablecoin payout responses mutex is poisoned")
            .pop_front()
            .unwrap_or_else(|| {
                Err(StablecoinPayoutClientError::Configuration(
                    "fake stablecoin payout client has no response configured".to_string(),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::Employee;
    use crate::domain::ids::StandardID;
    use crate::domain::payout_instruction::PayoutInstruction;
    use crate::domain::payrun::{CreatePayrunOptions, Payrun, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::{
        TokenSymbol, TreasuryAccount, TreasuryAccountDraft, TreasuryChain, TreasuryControlMode,
        TreasuryCustodyProvider,
    };
    use crate::domain::wallets::WalletAddress;
    use crate::services::stablecoin::client::{
        ProviderPayoutReference, StablecoinPayoutReviewReason, SubmittedStablecoinPayout,
    };

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn request() -> StablecoinPayoutRequest {
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
            custody_provider: TreasuryCustodyProvider::LocalKey,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: None,
            provider_owner_id: None,
            secret_reference: Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string()),
            is_default: true,
        })
        .unwrap();
        let instruction = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap();

        StablecoinPayoutRequest::from_instruction(&instruction)
    }

    #[tokio::test]
    async fn fake_client_returns_configured_response_and_records_request() {
        let outcome = StablecoinPayoutOutcome::Submitted(SubmittedStablecoinPayout::new(
            ProviderPayoutReference::parse("provider-1").unwrap(),
            None,
        ));
        let client = FakeStablecoinPayoutClient::with_success(outcome.clone());
        let request = request();

        let response = client.submit_payout(request.clone()).await.unwrap();

        assert_eq!(response, outcome);
        assert_eq!(client.requests(), vec![request]);
    }

    #[tokio::test]
    async fn fake_client_can_return_review_required() {
        let outcome = StablecoinPayoutOutcome::ReviewRequired(
            crate::services::stablecoin::client::ReviewRequiredStablecoinPayout::new(
                StablecoinPayoutReviewReason::AmbiguousProviderResponse,
                None,
            ),
        );
        let client = FakeStablecoinPayoutClient::with_success(outcome.clone());

        let response = client.submit_payout(request()).await.unwrap();

        assert_eq!(response, outcome);
    }

    #[tokio::test]
    async fn fake_client_errors_when_no_response_is_configured() {
        let client = FakeStablecoinPayoutClient::default();

        let err = client.submit_payout(request()).await.unwrap_err();

        assert!(matches!(err, StablecoinPayoutClientError::Configuration(_)));
    }
}
