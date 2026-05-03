use crate::domain::base_metadata::{LifecycleMeta, ObjectStatus};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TokenSymbol;
use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDCompensationProfile;

impl IDResource for IDCompensationProfile {
    fn prefix() -> Option<String> {
        Some("compensation".to_string())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct CompensationProfile {
    id: StandardID<IDCompensationProfile>,
    tenant_id: StandardID<IDTenant>,
    employee_id: StandardID<IDEmployee>,
    metadata: LifecycleMeta,
    amount: CompensationAmount,
    cadence: CompensationCadence,
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
}
impl CompensationProfile {
    pub fn new(draft: CompensationProfileDraft) -> Result<Self, CompensationProfileError> {
        let now = Utc::now();
        Self::restore(
            StandardID::new(),
            LifecycleMeta {
                status: ObjectStatus::Active,
                created: now,
                updated: now,
            },
            draft,
        )
    }

    pub fn restore(
        id: StandardID<IDCompensationProfile>,
        metadata: LifecycleMeta,
        mut draft: CompensationProfileDraft,
    ) -> Result<Self, CompensationProfileError> {
        draft.normalize();
        validate_amount(&draft.amount)?;
        validate_cadence(draft.cadence)?;
        validate_validity_window(draft.valid_from, draft.valid_to)?;

        Ok(Self {
            id,
            tenant_id: draft.tenant_id,
            employee_id: draft.employee_id,
            amount: draft.amount,
            cadence: draft.cadence,
            valid_from: draft.valid_from,
            valid_to: draft.valid_to,
            metadata,
        })
    }

    pub fn id(&self) -> &StandardID<IDCompensationProfile> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn employee_id(&self) -> &StandardID<IDEmployee> {
        &self.employee_id
    }

    pub fn metadata(&self) -> &LifecycleMeta {
        &self.metadata
    }

    pub fn status(&self) -> ObjectStatus {
        self.metadata.status
    }

    pub fn amount(&self) -> &CompensationAmount {
        &self.amount
    }

    pub fn cadence(&self) -> CompensationCadence {
        self.cadence
    }

    pub fn valid_from(&self) -> Option<DateTime<Utc>> {
        self.valid_from
    }

    pub fn valid_to(&self) -> Option<DateTime<Utc>> {
        self.valid_to
    }

    pub fn deactivate(&mut self) {
        self.metadata.status = ObjectStatus::Inactive;
        self.metadata.updated = Utc::now();
    }
}

impl CompensationProfileDraft {
    pub fn normalize(&mut self) {}
}
#[derive(Debug, Clone)]
pub struct CompensationProfileDraft {
    pub amount: CompensationAmount,
    pub tenant_id: StandardID<IDTenant>,
    pub employee_id: StandardID<IDEmployee>,
    pub cadence: CompensationCadence,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone)]
pub struct CompensationAmount {
    pub(crate) amount_units: u128,
    pub(crate) token_symbol: TokenSymbol,
}

impl Serialize for CompensationAmount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("CompensationAmount", 2)?;
        state.serialize_field("amount_units", &self.amount_units.to_string())?;
        state.serialize_field("token_symbol", &self.token_symbol)?;
        state.end()
    }
}
impl CompensationAmount {
    pub fn new(
        amount_units: u128,
        token_symbol: TokenSymbol,
    ) -> Result<Self, CompensationProfileError> {
        if amount_units == 0 {
            return Err(CompensationProfileError::InvalidAmountUnits);
        }

        Ok(Self {
            amount_units,
            token_symbol,
        })
    }

    pub fn amount_units(&self) -> u128 {
        self.amount_units
    }

    pub fn token_symbol(&self) -> &TokenSymbol {
        &self.token_symbol
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationCadence {
    Weekly,
    BiWeekly,
    Monthly,
    Custom { every: u16, unit: CadenceUnit },
}
impl CompensationCadence {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Weekly => "weekly",
            Self::BiWeekly => "biweekly",
            Self::Monthly => "monthly",
            Self::Custom { .. } => "custom",
        }
    }

    pub fn custom_every(&self) -> Option<u16> {
        match self {
            Self::Custom { every, .. } => Some(*every),
            _ => None,
        }
    }

    pub fn custom_unit(&self) -> Option<CadenceUnit> {
        match self {
            Self::Custom { unit, .. } => Some(*unit),
            _ => None,
        }
    }
}

impl Display for CompensationCadence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompensationCadence::Weekly => write!(f, "weekly"),
            CompensationCadence::BiWeekly => write!(f, "bi-weekly"),
            CompensationCadence::Monthly => write!(f, "monthly"),
            CompensationCadence::Custom { every, unit } => write!(f, "custom ({every} {unit})"),
        }
    }
}
impl CompensationCadence {
    pub fn parse(
        cadence: &str,
        every: Option<u16>,
        unit: Option<CadenceUnit>,
    ) -> Result<Self, CompensationProfileError> {
        match cadence.trim().to_ascii_lowercase().as_str() {
            "weekly" => Ok(Self::Weekly),
            "biweekly" | "bi-weekly" | "bi_weekly" => Ok(Self::BiWeekly),
            "monthly" => Ok(Self::Monthly),
            "custom" => {
                let every = every.ok_or(CompensationProfileError::InvalidCustomCadence)?;
                let unit = unit.ok_or(CompensationProfileError::InvalidCustomCadence)?;
                let cadence = Self::Custom { every, unit };
                validate_cadence(cadence)?;
                Ok(cadence)
            }
            _ => Err(CompensationProfileError::InvalidCustomCadence),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CadenceUnit {
    Days,
    Weeks,
    Months,
}

impl Display for CadenceUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CadenceUnit::Days => write!(f, "day"),
            CadenceUnit::Weeks => write!(f, "week"),
            CadenceUnit::Months => write!(f, "month"),
        }
    }
}

impl CadenceUnit {
    pub fn parse(raw: &str) -> Result<Self, CompensationProfileError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "day" | "days" => Ok(Self::Days),
            "week" | "weeks" => Ok(Self::Weeks),
            "month" | "months" => Ok(Self::Months),
            _ => Err(CompensationProfileError::InvalidCustomCadence),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompensationProfileError {
    #[error("compensation amount must be greater than 0")]
    InvalidAmountUnits,
    #[error("custom compensation cadence must repeat at least every 1 unit")]
    InvalidCustomCadence,
    #[error("compensation valid_to must be after valid_from")]
    InvalidValidityWindow,
}

fn validate_amount(amount: &CompensationAmount) -> Result<(), CompensationProfileError> {
    if amount.amount_units == 0 {
        return Err(CompensationProfileError::InvalidAmountUnits);
    }

    Ok(())
}

fn validate_cadence(cadence: CompensationCadence) -> Result<(), CompensationProfileError> {
    if let CompensationCadence::Custom { every: 0, .. } = cadence {
        return Err(CompensationProfileError::InvalidCustomCadence);
    }

    Ok(())
}

fn validate_validity_window(
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
) -> Result<(), CompensationProfileError> {
    if let (Some(valid_from), Some(valid_to)) = (valid_from, valid_to)
        && valid_to <= valid_from
    {
        return Err(CompensationProfileError::InvalidValidityWindow);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn amount() -> CompensationAmount {
        CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap()).unwrap()
    }

    fn draft() -> CompensationProfileDraft {
        CompensationProfileDraft {
            tenant_id: StandardID::new(),
            employee_id: StandardID::new(),
            amount: amount(),
            cadence: CompensationCadence::Monthly,
            valid_from: Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
            valid_to: None,
        }
    }

    #[test]
    fn creates_valid_profile() {
        let profile = CompensationProfile::new(draft()).unwrap();

        assert_eq!(profile.status(), ObjectStatus::Active);
        assert_eq!(profile.amount().amount_units(), 1_000_000);
        assert_eq!(profile.cadence(), CompensationCadence::Monthly);
    }

    #[test]
    fn rejects_zero_amount() {
        let err = CompensationAmount::new(0, TokenSymbol::parse("USDC").unwrap()).unwrap_err();

        assert_eq!(err, CompensationProfileError::InvalidAmountUnits);
    }

    #[test]
    fn rejects_zero_custom_cadence() {
        let mut draft = draft();
        draft.cadence = CompensationCadence::Custom {
            every: 0,
            unit: CadenceUnit::Weeks,
        };

        let err = CompensationProfile::new(draft).unwrap_err();

        assert_eq!(err, CompensationProfileError::InvalidCustomCadence);
    }

    #[test]
    fn rejects_invalid_validity_window() {
        let mut draft = draft();
        draft.valid_from = Some(Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap());
        draft.valid_to = Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap());

        let err = CompensationProfile::new(draft).unwrap_err();

        assert_eq!(err, CompensationProfileError::InvalidValidityWindow);
    }

    #[test]
    fn parses_standard_cadence() {
        let cadence = CompensationCadence::parse("bi-weekly", None, None).unwrap();

        assert_eq!(cadence, CompensationCadence::BiWeekly);
    }

    #[test]
    fn parses_custom_cadence() {
        let cadence =
            CompensationCadence::parse("custom", Some(2), Some(CadenceUnit::Weeks)).unwrap();

        assert_eq!(
            cadence,
            CompensationCadence::Custom {
                every: 2,
                unit: CadenceUnit::Weeks
            }
        );
    }

    #[test]
    fn rejects_invalid_custom_cadence_parts() {
        let err =
            CompensationCadence::parse("custom", Some(0), Some(CadenceUnit::Weeks)).unwrap_err();
        assert_eq!(err, CompensationProfileError::InvalidCustomCadence);

        let err = CompensationCadence::parse("custom", Some(2), None).unwrap_err();
        assert_eq!(err, CompensationProfileError::InvalidCustomCadence);
    }
}
