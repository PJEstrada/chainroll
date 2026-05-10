use crate::domain::treasury::{TreasuryChain, TreasuryControlMode, TreasuryCustodyProvider};
use crate::services::stablecoin::client::{
    ProviderPayoutReference, RejectedStablecoinPayout, ReviewRequiredStablecoinPayout,
    StablecoinPayoutClient, StablecoinPayoutClientError, StablecoinPayoutOutcome,
    StablecoinPayoutRejectionReason, StablecoinPayoutRequest, StablecoinPayoutReviewReason,
    SubmittedStablecoinPayout, TransactionHash,
};
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use chain_access::domain::chain_id::ChainId;
use chain_access::error::ChainAccessError;
use chain_access::ports::{ChainReader, ChainWriter};
use chain_access::signer::{PrivySigner, SignerBackend};
use privy_rs::PrivyClient;
use std::str::FromStr;

const PRIVY_APP_ID_ENV: &str = "PRIVY_APP_ID";
const PRIVY_APP_SECRET_ENV: &str = "PRIVY_APP_SECRET";

#[derive(Debug, Clone)]
pub struct TempoPrivyStablecoinPayoutClient {
    app_id: String,
    app_secret: String,
}

impl TempoPrivyStablecoinPayoutClient {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        }
    }

    pub fn from_env() -> Result<Self, StablecoinPayoutClientError> {
        let app_id = std::env::var(PRIVY_APP_ID_ENV).map_err(|_| {
            StablecoinPayoutClientError::Configuration(format!(
                "missing env var: {PRIVY_APP_ID_ENV}"
            ))
        })?;
        let app_secret = std::env::var(PRIVY_APP_SECRET_ENV).map_err(|_| {
            StablecoinPayoutClientError::Configuration(format!(
                "missing env var: {PRIVY_APP_SECRET_ENV}"
            ))
        })?;

        Ok(Self::new(app_id, app_secret))
    }

    fn privy_client(&self) -> Result<PrivyClient, StablecoinPayoutClientError> {
        PrivyClient::new(self.app_id.clone(), self.app_secret.clone())
            .map_err(|e| StablecoinPayoutClientError::Configuration(e.to_string()))
    }
}

impl StablecoinPayoutClient for TempoPrivyStablecoinPayoutClient {
    async fn submit_payout(
        &self,
        request: StablecoinPayoutRequest,
    ) -> Result<StablecoinPayoutOutcome, StablecoinPayoutClientError> {
        if let Some(outcome) = reject_unsupported_request(&request) {
            return Ok(outcome);
        }

        let provider_wallet_id = request.provider_wallet_id().ok_or_else(|| {
            StablecoinPayoutClientError::Configuration(
                "privy payout request is missing provider_wallet_id".to_string(),
            )
        })?;

        let reader = chain_access::connect_reader(ChainId::TempoTestnet)
            .await
            .map_err(map_chain_error)?;
        let writer = chain_access::connect_writer(ChainId::TempoTestnet)
            .await
            .map_err(map_chain_error)?;
        let signer = PrivySigner::new(self.privy_client()?, provider_wallet_id.to_string());

        submit_with_ports(&request, reader.as_ref(), writer.as_ref(), &signer).await
    }
}

async fn submit_with_ports(
    request: &StablecoinPayoutRequest,
    reader: &dyn ChainReader,
    writer: &dyn ChainWriter,
    signer: &dyn SignerBackend,
) -> Result<StablecoinPayoutOutcome, StablecoinPayoutClientError> {
    let source = match parse_address(request.source_wallet_address().as_str()) {
        Ok(source) => source,
        Err(_) => {
            return Ok(rejected(
                StablecoinPayoutRejectionReason::InvalidSourceWallet,
                None,
            ));
        }
    };
    let token = match parse_address(request.token_address().as_str()) {
        Ok(token) => token,
        Err(_) => {
            return Ok(rejected(
                StablecoinPayoutRejectionReason::UnsupportedToken,
                None,
            ));
        }
    };
    let destination = match parse_address(request.destination_wallet_address().as_str()) {
        Ok(destination) => destination,
        Err(_) => {
            return Ok(rejected(
                StablecoinPayoutRejectionReason::InvalidDestinationWallet,
                None,
            ));
        }
    };

    let signer_address = signer.address().await.map_err(map_chain_error)?;
    if signer_address != source {
        return Ok(rejected(
            StablecoinPayoutRejectionReason::InvalidSourceWallet,
            None,
        ));
    }

    let amount = U256::from(request.amount_units());
    let calldata = chain_access::domain::erc20::transfer_calldata(destination, amount);
    let nonce = reader.nonce(source).await.map_err(map_chain_error)?;
    let gas_price = reader.gas_price().await.map_err(map_chain_error)?;

    let tx = TransactionRequest::default()
        .with_from(source)
        .with_to(token)
        .with_input(calldata)
        .with_nonce(nonce)
        .with_chain_id(request.chain_id());

    let gas = reader.estimate_gas(&tx).await.map_err(map_chain_error)?;
    let full_tx = tx
        .with_gas_limit(gas)
        .with_max_fee_per_gas(gas_price)
        .with_max_priority_fee_per_gas(0);

    let raw = signer
        .sign_transaction(full_tx)
        .await
        .map_err(map_chain_error)?;
    let tx_hash = writer
        .send_raw_transaction(raw)
        .await
        .map_err(map_chain_error)?;
    let provider_reference = parse_provider_reference(tx_hash.to_string())?;
    let transaction_hash = parse_transaction_hash(tx_hash.to_string())?;

    match writer.wait_for_receipt(&tx_hash).await {
        Ok(receipt) if receipt.status() => Ok(StablecoinPayoutOutcome::Submitted(
            SubmittedStablecoinPayout::new(provider_reference, Some(transaction_hash)),
        )),
        Ok(_) => Ok(rejected(
            StablecoinPayoutRejectionReason::ProviderRejected,
            Some(provider_reference),
        )),
        Err(_) => Ok(StablecoinPayoutOutcome::ReviewRequired(
            ReviewRequiredStablecoinPayout::new(
                StablecoinPayoutReviewReason::TimeoutAfterSubmission,
                Some(provider_reference),
            ),
        )),
    }
}

fn reject_unsupported_request(
    request: &StablecoinPayoutRequest,
) -> Option<StablecoinPayoutOutcome> {
    if request.chain() != TreasuryChain::TempoTestnet || request.chain_id() != 42431 {
        return Some(rejected(
            StablecoinPayoutRejectionReason::UnsupportedChain,
            None,
        ));
    }

    if request.custody_provider() != TreasuryCustodyProvider::Privy {
        return Some(rejected(
            StablecoinPayoutRejectionReason::UnsupportedCustodyProvider,
            None,
        ));
    }

    if request.control_mode() != TreasuryControlMode::ServerControlled {
        return Some(rejected(
            StablecoinPayoutRejectionReason::UnsupportedControlMode,
            None,
        ));
    }

    None
}

fn rejected(
    reason: StablecoinPayoutRejectionReason,
    provider_reference: Option<ProviderPayoutReference>,
) -> StablecoinPayoutOutcome {
    StablecoinPayoutOutcome::Rejected(RejectedStablecoinPayout::new(reason, provider_reference))
}

fn parse_address(raw: &str) -> Result<Address, StablecoinPayoutClientError> {
    Address::from_str(raw).map_err(|e| StablecoinPayoutClientError::Serialization(e.to_string()))
}

fn parse_provider_reference(
    raw: impl AsRef<str>,
) -> Result<ProviderPayoutReference, StablecoinPayoutClientError> {
    ProviderPayoutReference::parse(raw)
        .map_err(|e| StablecoinPayoutClientError::Serialization(e.to_string()))
}

fn parse_transaction_hash(
    raw: impl AsRef<str>,
) -> Result<TransactionHash, StablecoinPayoutClientError> {
    TransactionHash::parse(raw)
        .map_err(|e| StablecoinPayoutClientError::Serialization(e.to_string()))
}

fn map_chain_error(err: ChainAccessError) -> StablecoinPayoutClientError {
    match err {
        ChainAccessError::UnsupportedChain(_) => {
            StablecoinPayoutClientError::Configuration(err.to_string())
        }
        ChainAccessError::Rpc(_) => {
            StablecoinPayoutClientError::ProviderUnavailable(err.to_string())
        }
        ChainAccessError::Signer(_) | ChainAccessError::TxBuild(_) | ChainAccessError::Other(_) => {
            StablecoinPayoutClientError::Transport(err.to_string())
        }
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
    use crate::domain::treasury::{TokenSymbol, TreasuryAccount, TreasuryAccountDraft};
    use crate::domain::wallets::WalletAddress;

    fn wallet(raw: &str) -> WalletAddress {
        WalletAddress::parse(raw).unwrap()
    }

    fn request_with_treasury(
        custody_provider: TreasuryCustodyProvider,
        control_mode: TreasuryControlMode,
        provider_wallet_id: Option<String>,
    ) -> StablecoinPayoutRequest {
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
            custody_provider,
            control_mode,
            provider_wallet_id,
            provider_owner_id: if control_mode == TreasuryControlMode::UserDelegated {
                Some("privy-user-id".to_string())
            } else {
                None
            },
            secret_reference: if custody_provider == TreasuryCustodyProvider::LocalKey {
                Some("env:TEMPO_TREASURY_PRIVATE_KEY".to_string())
            } else {
                None
            },
            is_default: true,
        })
        .unwrap();
        let instruction = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap();

        StablecoinPayoutRequest::from_instruction(&instruction)
    }

    #[tokio::test]
    async fn rejects_unsupported_custody_provider_without_network_call() {
        let client = TempoPrivyStablecoinPayoutClient::new("app-id", "secret");
        let request = request_with_treasury(
            TreasuryCustodyProvider::LocalKey,
            TreasuryControlMode::ServerControlled,
            None,
        );

        let outcome = client.submit_payout(request).await.unwrap();

        assert!(matches!(
            outcome,
            StablecoinPayoutOutcome::Rejected(rejected)
                if rejected.reason() == &StablecoinPayoutRejectionReason::UnsupportedCustodyProvider
        ));
    }

    #[tokio::test]
    async fn rejects_unsupported_control_mode_without_network_call() {
        let client = TempoPrivyStablecoinPayoutClient::new("app-id", "secret");
        let request = request_with_treasury(
            TreasuryCustodyProvider::Privy,
            TreasuryControlMode::UserDelegated,
            Some("privy-wallet-id".to_string()),
        );

        let outcome = client.submit_payout(request).await.unwrap();

        assert!(matches!(
            outcome,
            StablecoinPayoutOutcome::Rejected(rejected)
                if rejected.reason() == &StablecoinPayoutRejectionReason::UnsupportedControlMode
        ));
    }

    #[tokio::test]
    async fn missing_provider_wallet_id_is_configuration_error() {
        let client = TempoPrivyStablecoinPayoutClient::new("app-id", "secret");
        let request = request_with_treasury(
            TreasuryCustodyProvider::Privy,
            TreasuryControlMode::ServerControlled,
            Some("privy-wallet-id".to_string()),
        );
        let request = request.without_provider_wallet_id();

        let err = client.submit_payout(request).await.unwrap_err();

        assert!(matches!(err, StablecoinPayoutClientError::Configuration(_)));
    }

    #[tokio::test]
    #[ignore = "requires funded Tempo testnet Privy wallet"]
    async fn tempo_privy_real_payout() {
        let app_id = std::env::var(PRIVY_APP_ID_ENV).expect("missing PRIVY_APP_ID");
        let app_secret = std::env::var(PRIVY_APP_SECRET_ENV).expect("missing PRIVY_APP_SECRET");
        let wallet_id =
            std::env::var("TEMPO_PRIVY_WALLET_ID").expect("missing TEMPO_PRIVY_WALLET_ID");
        let privy = PrivyClient::new(app_id.clone(), app_secret.clone()).unwrap();
        let wallet_address = privy
            .wallets()
            .get(&wallet_id)
            .await
            .unwrap()
            .address
            .clone();
        let wallet_address = WalletAddress::parse(wallet_address).unwrap();

        let payrun = Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::new(),
                    CompensationAmount::new(1, TokenSymbol::parse("pathUSD").unwrap()).unwrap(),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap();
        let item = &payrun.items()[0];
        let employee = Employee::new("EMP-001".to_string(), "Jane".to_string(), "Doe".to_string())
            .with_id(*item.employee_id())
            .with_wallet_address(Some(wallet_address.clone()));
        let treasury = TreasuryAccount::new(TreasuryAccountDraft {
            tenant_id: *payrun.tenant_id(),
            name: "Tempo payout source".to_string(),
            chain: TreasuryChain::TempoTestnet,
            token_symbol: TokenSymbol::parse("pathUSD").unwrap(),
            token_address: wallet("0x20c0000000000000000000000000000000000000"),
            token_decimals: 18,
            sender_address: wallet_address,
            custody_provider: TreasuryCustodyProvider::Privy,
            control_mode: TreasuryControlMode::ServerControlled,
            provider_wallet_id: Some(wallet_id),
            provider_owner_id: None,
            secret_reference: None,
            is_default: true,
        })
        .unwrap();
        let instruction = PayoutInstruction::new(&payrun, item, &employee, &treasury).unwrap();
        let client = TempoPrivyStablecoinPayoutClient::new(app_id, app_secret);

        let outcome = client
            .submit_payout(StablecoinPayoutRequest::from_instruction(&instruction))
            .await
            .unwrap();

        assert!(matches!(
            outcome,
            StablecoinPayoutOutcome::Submitted(_) | StablecoinPayoutOutcome::ReviewRequired(_)
        ));
    }
}
