use crate::domain::ids::{IDResource, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::user::IDUser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    id: StandardID<IDAudit>,
    tenant_id: StandardID<IDTenant>,
    actor_id: StandardID<IDUser>,
    entity_type: AuditEntityType,
    entity_id: String,
    event_type: AuditEventType,
    payload: Value,
    created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        tenant_id: StandardID<IDTenant>,
        actor_id: StandardID<IDUser>,
        entity_type: AuditEntityType,
        entity_id: impl Into<String>,
        event_type: AuditEventType,
        payload: Value,
    ) -> Self {
        Self {
            id: StandardID::new(),
            tenant_id,
            actor_id,
            entity_type,
            entity_id: entity_id.into(),
            event_type,
            payload,
            created_at: Utc::now(),
        }
    }

    pub fn with_id(mut self, id: StandardID<IDAudit>) -> Self {
        self.id = id;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn id(&self) -> &StandardID<IDAudit> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn actor_id(&self) -> &StandardID<IDUser> {
        &self.actor_id
    }

    pub fn entity_type(&self) -> AuditEntityType {
        self.entity_type
    }

    pub fn entity_id(&self) -> &str {
        &self.entity_id
    }

    pub fn event_type(&self) -> AuditEventType {
        self.event_type
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDAudit;

impl IDResource for IDAudit {
    fn prefix() -> Option<String> {
        Some("audit_event".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEntityType {
    Employee,
    TreasuryAccount,
    CompensationProfile,
    Payrun,
    PayoutInstruction,
    PayoutAttempt,
}

impl Display for AuditEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEntityType::Employee => write!(f, "employee"),
            AuditEntityType::TreasuryAccount => write!(f, "treasury_account"),
            AuditEntityType::CompensationProfile => write!(f, "compensation_profile"),
            AuditEntityType::Payrun => write!(f, "payrun"),
            AuditEntityType::PayoutInstruction => write!(f, "payout_instruction"),
            AuditEntityType::PayoutAttempt => write!(f, "payout_attempt"),
        }
    }
}

impl FromStr for AuditEntityType {
    type Err = ParseAuditEntityTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "employee" => Ok(Self::Employee),
            "treasury_account" => Ok(Self::TreasuryAccount),
            "compensation_profile" => Ok(Self::CompensationProfile),
            "payrun" => Ok(Self::Payrun),
            "payout_instruction" => Ok(Self::PayoutInstruction),
            "payout_attempt" => Ok(Self::PayoutAttempt),
            other => Err(ParseAuditEntityTypeError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid audit entity type: {0}")]
pub struct ParseAuditEntityTypeError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    EmployeeCreated,
    TreasuryAccountCreated,
    TreasuryAccountUpdated,
    TreasuryAccountDeactivated,
    CompensationProfileCreated,
    CompensationProfileUpdated,
    PayrunCreated,
    PayoutAttemptStarted,
    PayoutAttemptSubmitted,
    PayoutAttemptFailed,
    PayoutAttemptReviewRequired,
}

impl Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::EmployeeCreated => write!(f, "employee_created"),
            AuditEventType::TreasuryAccountCreated => write!(f, "treasury_account_created"),
            AuditEventType::TreasuryAccountUpdated => write!(f, "treasury_account_updated"),
            AuditEventType::TreasuryAccountDeactivated => {
                write!(f, "treasury_account_deactivated")
            }
            AuditEventType::CompensationProfileCreated => write!(f, "compensation_profile_created"),
            AuditEventType::CompensationProfileUpdated => write!(f, "compensation_profile_updated"),
            AuditEventType::PayrunCreated => write!(f, "payrun_created"),
            AuditEventType::PayoutAttemptStarted => write!(f, "payout_attempt_started"),
            AuditEventType::PayoutAttemptSubmitted => write!(f, "payout_attempt_submitted"),
            AuditEventType::PayoutAttemptFailed => write!(f, "payout_attempt_failed"),
            AuditEventType::PayoutAttemptReviewRequired => {
                write!(f, "payout_attempt_review_required")
            }
        }
    }
}

impl FromStr for AuditEventType {
    type Err = ParseAuditEventTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "employee_created" => Ok(Self::EmployeeCreated),
            "treasury_account_created" => Ok(Self::TreasuryAccountCreated),
            "treasury_account_updated" => Ok(Self::TreasuryAccountUpdated),
            "treasury_account_deactivated" => Ok(Self::TreasuryAccountDeactivated),
            "compensation_profile_created" => Ok(Self::CompensationProfileCreated),
            "compensation_profile_updated" => Ok(Self::CompensationProfileUpdated),
            "payrun_created" => Ok(Self::PayrunCreated),
            "payout_attempt_started" => Ok(Self::PayoutAttemptStarted),
            "payout_attempt_submitted" => Ok(Self::PayoutAttemptSubmitted),
            "payout_attempt_failed" => Ok(Self::PayoutAttemptFailed),
            "payout_attempt_review_required" => Ok(Self::PayoutAttemptReviewRequired),
            other => Err(ParseAuditEventTypeError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid audit event type: {0}")]
pub struct ParseAuditEventTypeError(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entity_type_from_database_value() {
        assert_eq!(
            "treasury_account".parse::<AuditEntityType>().unwrap(),
            AuditEntityType::TreasuryAccount
        );
        assert!("unknown".parse::<AuditEntityType>().is_err());
    }

    #[test]
    fn parses_event_type_from_database_value() {
        assert_eq!(
            "payrun_created".parse::<AuditEventType>().unwrap(),
            AuditEventType::PayrunCreated
        );
        assert!("unknown".parse::<AuditEventType>().is_err());
    }
}
