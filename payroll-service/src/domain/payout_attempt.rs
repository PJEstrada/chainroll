use crate::domain::ids::{IDResource, StandardID};
use crate::domain::payout_instruction::IDPayoutInstruction;
use crate::domain::payrun::IDPayrun;
use crate::domain::tenant::IDTenant;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

const MAX_PROVIDER_REFERENCE_LEN: usize = 200;
const MAX_TRANSACTION_HASH_LEN: usize = 66;
const MAX_ERROR_MESSAGE_LEN: usize = 1_000;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDPayoutAttempt;

impl IDResource for IDPayoutAttempt {
    fn prefix() -> Option<String> {
        Some("payout_attempt".to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PayoutAttempt {
    id: StandardID<IDPayoutAttempt>,
    tenant_id: StandardID<IDTenant>,
    payrun_id: StandardID<IDPayrun>,
    payout_instruction_id: StandardID<IDPayoutInstruction>,
    attempt_number: u32,
    status: PayoutAttemptStatus,
    provider: PayoutProvider,
    signer_provider: PayoutSignerProvider,
    provider_reference: Option<String>,
    transaction_hash: Option<String>,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl PayoutAttempt {
    pub fn started(
        tenant_id: StandardID<IDTenant>,
        payrun_id: StandardID<IDPayrun>,
        payout_instruction_id: StandardID<IDPayoutInstruction>,
        attempt_number: u32,
    ) -> Result<Self, PayoutAttemptError> {
        Self::restore(
            StandardID::new(),
            PayoutAttemptDraft {
                tenant_id,
                payrun_id,
                payout_instruction_id,
                attempt_number,
                status: PayoutAttemptStatus::Started,
                provider: PayoutProvider::Tempo,
                signer_provider: PayoutSignerProvider::Privy,
                provider_reference: None,
                transaction_hash: None,
                error_message: None,
                started_at: Utc::now(),
                completed_at: None,
            },
        )
    }

    pub fn restore(
        id: StandardID<IDPayoutAttempt>,
        mut draft: PayoutAttemptDraft,
    ) -> Result<Self, PayoutAttemptError> {
        draft.normalize();
        validate_attempt_number(draft.attempt_number)?;
        validate_provider_reference(draft.provider_reference.as_deref())?;
        validate_transaction_hash(draft.transaction_hash.as_deref())?;
        validate_error_message(draft.error_message.as_deref())?;
        validate_status_fields(&draft)?;

        Ok(Self {
            id,
            tenant_id: draft.tenant_id,
            payrun_id: draft.payrun_id,
            payout_instruction_id: draft.payout_instruction_id,
            attempt_number: draft.attempt_number,
            status: draft.status,
            provider: draft.provider,
            signer_provider: draft.signer_provider,
            provider_reference: draft.provider_reference,
            transaction_hash: draft.transaction_hash,
            error_message: draft.error_message,
            started_at: draft.started_at,
            completed_at: draft.completed_at,
        })
    }

    pub fn mark_submitted(
        &self,
        provider_reference: impl Into<String>,
        transaction_hash: Option<String>,
    ) -> Result<Self, PayoutAttemptError> {
        self.transition(
            PayoutAttemptStatus::Submitted,
            Some(provider_reference.into()),
            transaction_hash,
            None,
        )
    }

    pub fn mark_failed(
        &self,
        error_message: impl Into<String>,
        provider_reference: Option<String>,
    ) -> Result<Self, PayoutAttemptError> {
        self.transition(
            PayoutAttemptStatus::Failed,
            provider_reference,
            None,
            Some(error_message.into()),
        )
    }

    pub fn mark_review_required(
        &self,
        error_message: impl Into<String>,
        provider_reference: Option<String>,
    ) -> Result<Self, PayoutAttemptError> {
        self.transition(
            PayoutAttemptStatus::ReviewRequired,
            provider_reference,
            None,
            Some(error_message.into()),
        )
    }

    fn transition(
        &self,
        status: PayoutAttemptStatus,
        provider_reference: Option<String>,
        transaction_hash: Option<String>,
        error_message: Option<String>,
    ) -> Result<Self, PayoutAttemptError> {
        if self.status != PayoutAttemptStatus::Started {
            return Err(PayoutAttemptError::AttemptAlreadyFinalized);
        }

        Self::restore(
            self.id,
            PayoutAttemptDraft {
                tenant_id: self.tenant_id,
                payrun_id: self.payrun_id,
                payout_instruction_id: self.payout_instruction_id,
                attempt_number: self.attempt_number,
                status,
                provider: self.provider,
                signer_provider: self.signer_provider,
                provider_reference,
                transaction_hash,
                error_message,
                started_at: self.started_at,
                completed_at: Some(Utc::now()),
            },
        )
    }

    pub fn id(&self) -> &StandardID<IDPayoutAttempt> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn payrun_id(&self) -> &StandardID<IDPayrun> {
        &self.payrun_id
    }

    pub fn payout_instruction_id(&self) -> &StandardID<IDPayoutInstruction> {
        &self.payout_instruction_id
    }

    pub fn attempt_number(&self) -> u32 {
        self.attempt_number
    }

    pub fn status(&self) -> PayoutAttemptStatus {
        self.status
    }

    pub fn provider(&self) -> PayoutProvider {
        self.provider
    }

    pub fn signer_provider(&self) -> PayoutSignerProvider {
        self.signer_provider
    }

    pub fn provider_reference(&self) -> Option<&str> {
        self.provider_reference.as_deref()
    }

    pub fn transaction_hash(&self) -> Option<&str> {
        self.transaction_hash.as_deref()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
}

#[derive(Debug, Clone)]
pub struct PayoutAttemptDraft {
    pub tenant_id: StandardID<IDTenant>,
    pub payrun_id: StandardID<IDPayrun>,
    pub payout_instruction_id: StandardID<IDPayoutInstruction>,
    pub attempt_number: u32,
    pub status: PayoutAttemptStatus,
    pub provider: PayoutProvider,
    pub signer_provider: PayoutSignerProvider,
    pub provider_reference: Option<String>,
    pub transaction_hash: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl PayoutAttemptDraft {
    fn normalize(&mut self) {
        self.provider_reference = normalize_optional_string(self.provider_reference.take());
        self.transaction_hash = normalize_optional_string(self.transaction_hash.take());
        self.error_message = normalize_optional_string(self.error_message.take());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutAttemptStatus {
    Started,
    Submitted,
    Failed,
    ReviewRequired,
}

impl Display for PayoutAttemptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayoutAttemptStatus::Started => write!(f, "started"),
            PayoutAttemptStatus::Submitted => write!(f, "submitted"),
            PayoutAttemptStatus::Failed => write!(f, "failed"),
            PayoutAttemptStatus::ReviewRequired => write!(f, "review_required"),
        }
    }
}

impl FromStr for PayoutAttemptStatus {
    type Err = ParsePayoutAttemptStatusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "started" => Ok(Self::Started),
            "submitted" => Ok(Self::Submitted),
            "failed" => Ok(Self::Failed),
            "review_required" => Ok(Self::ReviewRequired),
            other => Err(ParsePayoutAttemptStatusError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid payout attempt status: {0}")]
pub struct ParsePayoutAttemptStatusError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutProvider {
    Tempo,
}

impl Display for PayoutProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayoutProvider::Tempo => write!(f, "tempo"),
        }
    }
}

impl FromStr for PayoutProvider {
    type Err = ParsePayoutProviderError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tempo" => Ok(Self::Tempo),
            other => Err(ParsePayoutProviderError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid payout provider: {0}")]
pub struct ParsePayoutProviderError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutSignerProvider {
    Privy,
}

impl Display for PayoutSignerProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayoutSignerProvider::Privy => write!(f, "privy"),
        }
    }
}

impl FromStr for PayoutSignerProvider {
    type Err = ParsePayoutSignerProviderError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "privy" => Ok(Self::Privy),
            other => Err(ParsePayoutSignerProviderError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid payout signer provider: {0}")]
pub struct ParsePayoutSignerProviderError(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayoutAttemptError {
    #[error("attempt number must be greater than zero")]
    InvalidAttemptNumber,
    #[error("provider reference cannot be longer than {max} characters")]
    ProviderReferenceTooLong { max: usize },
    #[error("transaction hash must be a 0x-prefixed 32-byte hex value")]
    InvalidTransactionHash,
    #[error("error message cannot be longer than {max} characters")]
    ErrorMessageTooLong { max: usize },
    #[error("started payout attempts cannot have completion fields")]
    StartedAttemptHasCompletionFields,
    #[error("final payout attempts require completed_at")]
    FinalAttemptRequiresCompletedAt,
    #[error("submitted payout attempts require provider_reference")]
    SubmittedAttemptRequiresProviderReference,
    #[error("failed payout attempts require error_message")]
    FailedAttemptRequiresErrorMessage,
    #[error("review-required payout attempts require error_message")]
    ReviewRequiredAttemptRequiresErrorMessage,
    #[error("payout attempt is already finalized")]
    AttemptAlreadyFinalized,
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn validate_attempt_number(attempt_number: u32) -> Result<(), PayoutAttemptError> {
    if attempt_number == 0 {
        return Err(PayoutAttemptError::InvalidAttemptNumber);
    }

    Ok(())
}

fn validate_provider_reference(value: Option<&str>) -> Result<(), PayoutAttemptError> {
    if let Some(value) = value
        && value.len() > MAX_PROVIDER_REFERENCE_LEN
    {
        return Err(PayoutAttemptError::ProviderReferenceTooLong {
            max: MAX_PROVIDER_REFERENCE_LEN,
        });
    }

    Ok(())
}

fn validate_transaction_hash(value: Option<&str>) -> Result<(), PayoutAttemptError> {
    let Some(value) = value else {
        return Ok(());
    };
    let hash = value
        .strip_prefix("0x")
        .ok_or(PayoutAttemptError::InvalidTransactionHash)?;

    if value.len() > MAX_TRANSACTION_HASH_LEN
        || hash.len() != 64
        || !hash.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err(PayoutAttemptError::InvalidTransactionHash);
    }

    Ok(())
}

fn validate_error_message(value: Option<&str>) -> Result<(), PayoutAttemptError> {
    if let Some(value) = value
        && value.len() > MAX_ERROR_MESSAGE_LEN
    {
        return Err(PayoutAttemptError::ErrorMessageTooLong {
            max: MAX_ERROR_MESSAGE_LEN,
        });
    }

    Ok(())
}

fn validate_status_fields(draft: &PayoutAttemptDraft) -> Result<(), PayoutAttemptError> {
    match draft.status {
        PayoutAttemptStatus::Started => {
            if draft.completed_at.is_some()
                || draft.provider_reference.is_some()
                || draft.transaction_hash.is_some()
                || draft.error_message.is_some()
            {
                return Err(PayoutAttemptError::StartedAttemptHasCompletionFields);
            }
        }
        PayoutAttemptStatus::Submitted => {
            if draft.completed_at.is_none() {
                return Err(PayoutAttemptError::FinalAttemptRequiresCompletedAt);
            }
            if draft.provider_reference.is_none() {
                return Err(PayoutAttemptError::SubmittedAttemptRequiresProviderReference);
            }
        }
        PayoutAttemptStatus::Failed => {
            if draft.completed_at.is_none() {
                return Err(PayoutAttemptError::FinalAttemptRequiresCompletedAt);
            }
            if draft.error_message.is_none() {
                return Err(PayoutAttemptError::FailedAttemptRequiresErrorMessage);
            }
        }
        PayoutAttemptStatus::ReviewRequired => {
            if draft.completed_at.is_none() {
                return Err(PayoutAttemptError::FinalAttemptRequiresCompletedAt);
            }
            if draft.error_message.is_none() {
                return Err(PayoutAttemptError::ReviewRequiredAttemptRequiresErrorMessage);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started_attempt() -> PayoutAttempt {
        PayoutAttempt::started(StandardID::new(), StandardID::new(), StandardID::new(), 1).unwrap()
    }

    #[test]
    fn creates_started_attempt() {
        let attempt = started_attempt();

        assert_eq!(attempt.status(), PayoutAttemptStatus::Started);
        assert_eq!(attempt.provider(), PayoutProvider::Tempo);
        assert_eq!(attempt.signer_provider(), PayoutSignerProvider::Privy);
        assert!(attempt.completed_at().is_none());
    }

    #[test]
    fn submitted_attempt_requires_provider_reference() {
        let err = PayoutAttempt::restore(
            StandardID::new(),
            PayoutAttemptDraft {
                tenant_id: StandardID::new(),
                payrun_id: StandardID::new(),
                payout_instruction_id: StandardID::new(),
                attempt_number: 1,
                status: PayoutAttemptStatus::Submitted,
                provider: PayoutProvider::Tempo,
                signer_provider: PayoutSignerProvider::Privy,
                provider_reference: None,
                transaction_hash: None,
                error_message: None,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            PayoutAttemptError::SubmittedAttemptRequiresProviderReference
        );
    }

    #[test]
    fn marks_attempt_submitted() {
        let hash = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        let attempt = started_attempt()
            .mark_submitted(hash.clone(), Some(hash.clone()))
            .unwrap();

        assert_eq!(attempt.status(), PayoutAttemptStatus::Submitted);
        assert_eq!(attempt.provider_reference(), Some(hash.as_str()));
        assert_eq!(attempt.transaction_hash(), Some(hash.as_str()));
        assert!(attempt.completed_at().is_some());
    }

    #[test]
    fn final_attempt_cannot_transition_again() {
        let attempt = started_attempt()
            .mark_failed("provider rejected", None)
            .unwrap();

        let err = attempt.mark_review_required("ambiguous", None).unwrap_err();

        assert_eq!(err, PayoutAttemptError::AttemptAlreadyFinalized);
    }

    #[test]
    fn parses_status_provider_and_signer_provider() {
        assert_eq!(
            "review_required".parse::<PayoutAttemptStatus>().unwrap(),
            PayoutAttemptStatus::ReviewRequired
        );
        assert_eq!(
            "tempo".parse::<PayoutProvider>().unwrap(),
            PayoutProvider::Tempo
        );
        assert_eq!(
            "privy".parse::<PayoutSignerProvider>().unwrap(),
            PayoutSignerProvider::Privy
        );
    }
}
