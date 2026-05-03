use crate::Result;
use crate::domain::audit::{AuditEntityType, AuditEvent, AuditEventType};
use crate::domain::compensation::{CompensationProfile, IDCompensationProfile};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::user::IDUser;
use crate::error::Error;
use crate::services::compensation::create::{CompensationProfileData, draft_from_data};
use crate::services::compensation::service::{CompensationServiceImpl, map_store_error};
use crate::services::datastore::CompensationStore;
use error_stack::ResultExt;
use serde_json::json;

pub struct UpdateRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub employee_id: StandardID<IDEmployee>,
    pub actor_id: StandardID<IDUser>,
    pub id: StandardID<IDCompensationProfile>,
    pub data: CompensationProfileData,
}

#[derive(Debug)]
pub struct UpdateResponse {
    pub compensation_profile: CompensationProfile,
}

pub(super) async fn execute<S: CompensationStore>(
    svc: &CompensationServiceImpl<S>,
    req: UpdateRequest,
) -> Result<UpdateResponse> {
    let existing = svc
        .store()
        .get(&req.id)
        .await
        .map_err(map_store_error)?
        .filter(|profile| {
            profile.tenant_id() == &req.tenant_id && profile.employee_id() == &req.employee_id
        })
        .ok_or(Error::NotFound)?;

    let profile = CompensationProfile::restore(
        req.id,
        existing.metadata().clone(),
        draft_from_data(req.tenant_id, req.employee_id, req.data)?,
    )
    .change_context(Error::InvalidInput(
        "invalid compensation profile".to_string(),
    ))?;

    let audit_event = AuditEvent::new(
        *profile.tenant_id(),
        req.actor_id,
        AuditEntityType::CompensationProfile,
        profile.id().to_string(),
        AuditEventType::CompensationProfileUpdated,
        json!({ "compensation_profile": profile }),
    );

    let compensation_profile = svc
        .store()
        .update(&profile, &audit_event)
        .await
        .map_err(map_store_error)?;

    Ok(UpdateResponse {
        compensation_profile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::{
        CompensationAmount, CompensationCadence, CompensationProfileDraft,
    };
    use crate::domain::treasury::TokenSymbol;
    use crate::services::datastore::MockCompensationStore;

    fn profile_for(
        tenant_id: StandardID<IDTenant>,
        employee_id: StandardID<IDEmployee>,
        amount_units: u128,
    ) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id,
            employee_id,
            amount: CompensationAmount::new(amount_units, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    fn data(amount_units: &str) -> CompensationProfileData {
        CompensationProfileData {
            amount_units: amount_units.to_string(),
            token_symbol: "USDC".to_string(),
            cadence: "monthly".to_string(),
            cadence_every: None,
            cadence_unit: None,
            valid_from: None,
            valid_to: None,
        }
    }

    #[tokio::test]
    async fn updates_compensation_profile_with_audit_event() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let existing = profile_for(tenant_id, employee_id, 1_000_000);
        let id = *existing.id();
        let existing_for_get = existing.clone();
        let mut store = MockCompensationStore::new();
        store
            .expect_get()
            .withf(move |actual_id| actual_id == &id)
            .returning(move |_| Ok(Some(existing_for_get.clone())));
        store
            .expect_update()
            .withf(|profile, audit_event| {
                audit_event.entity_type() == AuditEntityType::CompensationProfile
                    && audit_event.event_type() == AuditEventType::CompensationProfileUpdated
                    && audit_event.entity_id() == profile.id().to_string()
                    && profile.amount().amount_units() == 2_000_000
            })
            .returning(|profile, _| Ok(profile.clone()));

        let svc = CompensationServiceImpl::new(store);
        let response = execute(
            &svc,
            UpdateRequest {
                tenant_id,
                employee_id,
                actor_id: StandardID::new(),
                id,
                data: data("2000000"),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            response.compensation_profile.amount().amount_units(),
            2_000_000
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_profile_belongs_to_another_employee() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let existing = profile_for(tenant_id, StandardID::new(), 1_000_000);
        let id = *existing.id();
        let mut store = MockCompensationStore::new();
        store
            .expect_get()
            .returning(move |_| Ok(Some(existing.clone())));

        let svc = CompensationServiceImpl::new(store);
        let result = execute(
            &svc,
            UpdateRequest {
                tenant_id,
                employee_id,
                actor_id: StandardID::new(),
                id,
                data: data("2000000"),
            },
        )
        .await;

        assert!(matches!(
            result.unwrap_err().current_context(),
            Error::NotFound
        ));
    }
}
