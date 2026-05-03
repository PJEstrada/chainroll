use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::compensation::{
    CadenceUnit, CompensationAmount, CompensationCadence, CompensationProfile,
    CompensationProfileDraft,
};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TokenSymbol;
use crate::domain::user::IDUser;
use crate::error::Error;
use crate::services::compensation::service::{CompensationServiceImpl, map_store_error};
use crate::services::datastore::CompensationStore;
use chrono::{DateTime, Utc};
use error_stack::ResultExt;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
pub struct CompensationProfileData {
    pub amount_units: String,
    pub token_symbol: String,
    pub cadence: String,
    pub cadence_every: Option<u16>,
    pub cadence_unit: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

pub struct CreateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub employee_id: StandardID<IDEmployee>,
    pub actor_id: StandardID<IDUser>,
    pub data: CompensationProfileData,
}

#[derive(Debug)]
pub struct CreateResponse {
    pub compensation_profile: CompensationProfile,
}

pub(super) async fn execute<S: CompensationStore>(
    svc: &CompensationServiceImpl<S>,
    req: CreateRequest,
) -> Result<CreateResponse> {
    let profile =
        CompensationProfile::new(draft_from_data(req.tenant_id, req.employee_id, req.data)?)
            .change_context(Error::InvalidInput(
                "invalid compensation profile".to_string(),
            ))?;

    let audit_event = AuditEvent::new(
        *profile.tenant_id(),
        req.actor_id,
        AuditEntityType::CompensationProfile,
        profile.id().to_string(),
        AuditEventType::CompensationProfileCreated,
        json!({ "compensation_profile": profile }),
    );

    let compensation_profile = svc
        .store()
        .create(&profile, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(CreateResponse {
        compensation_profile,
    })
}

pub(super) fn draft_from_data(
    tenant_id: StandardID<IDTenant>,
    employee_id: StandardID<IDEmployee>,
    data: CompensationProfileData,
) -> Result<CompensationProfileDraft> {
    let amount_units = data
        .amount_units
        .trim()
        .parse::<u128>()
        .change_context(Error::InvalidInput("invalid amount units".to_string()))?;
    let token_symbol = TokenSymbol::parse(data.token_symbol)
        .change_context(Error::InvalidInput("invalid token symbol".to_string()))?;
    let cadence_unit = data
        .cadence_unit
        .as_deref()
        .map(CadenceUnit::parse)
        .transpose()
        .change_context(Error::InvalidInput("invalid cadence unit".to_string()))?;
    let cadence = CompensationCadence::parse(&data.cadence, data.cadence_every, cadence_unit)
        .change_context(Error::InvalidInput(
            "invalid compensation cadence".to_string(),
        ))?;
    let amount = CompensationAmount::new(amount_units, token_symbol).change_context(
        Error::InvalidInput("invalid compensation amount".to_string()),
    )?;

    Ok(CompensationProfileDraft {
        tenant_id,
        employee_id,
        amount,
        cadence,
        valid_from: data.valid_from,
        valid_to: data.valid_to,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::treasury::TokenSymbol;
    use crate::services::datastore::MockCompensationStore;

    fn data() -> CompensationProfileData {
        CompensationProfileData {
            amount_units: "1000000".to_string(),
            token_symbol: "USDC".to_string(),
            cadence: "monthly".to_string(),
            cadence_every: None,
            cadence_unit: None,
            valid_from: None,
            valid_to: None,
        }
    }

    fn request() -> CreateRequest {
        CreateRequest {
            tenant_id: StandardID::new(),
            employee_id: StandardID::new(),
            actor_id: StandardID::new(),
            data: data(),
        }
    }

    #[tokio::test]
    async fn creates_compensation_profile_with_audit_event() {
        let mut store = MockCompensationStore::new();
        store
            .expect_create()
            .withf(|profile, audit_event| {
                audit_event.entity_type() == AuditEntityType::CompensationProfile
                    && audit_event.event_type() == AuditEventType::CompensationProfileCreated
                    && audit_event.entity_id() == profile.id().to_string()
                    && profile.amount().token_symbol() == &TokenSymbol::parse("USDC").unwrap()
            })
            .returning(|profile, _| Ok(profile.clone()));

        let svc = CompensationServiceImpl::new(store);
        let response = execute(&svc, request()).await.unwrap();

        assert_eq!(
            response.compensation_profile.amount().amount_units(),
            1_000_000
        );
        assert_eq!(
            response.compensation_profile.cadence(),
            CompensationCadence::Monthly
        );
    }

    #[tokio::test]
    async fn returns_invalid_input_for_bad_amount() {
        let store = MockCompensationStore::new();
        let svc = CompensationServiceImpl::new(store);
        let mut req = request();
        req.data.amount_units = "bad".to_string();

        let result = execute(&svc, req).await;

        assert!(matches!(
            result.unwrap_err().current_context(),
            Error::InvalidInput(_)
        ));
    }

    #[tokio::test]
    async fn returns_invalid_input_for_bad_custom_cadence() {
        let store = MockCompensationStore::new();
        let svc = CompensationServiceImpl::new(store);
        let mut req = request();
        req.data.cadence = "custom".to_string();
        req.data.cadence_every = Some(0);
        req.data.cadence_unit = Some("weeks".to_string());

        let result = execute(&svc, req).await;

        assert!(matches!(
            result.unwrap_err().current_context(),
            Error::InvalidInput(_)
        ));
    }
}
