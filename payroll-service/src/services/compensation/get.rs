use crate::Result;
use crate::domain::compensation::{CompensationProfile, IDCompensationProfile};
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::compensation::service::{CompensationServiceImpl, map_store_error};
use crate::services::datastore::CompensationStore;

pub struct GetRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub employee_id: StandardID<IDEmployee>,
    pub id: StandardID<IDCompensationProfile>,
}

pub struct GetResponse {
    pub compensation_profile: Option<CompensationProfile>,
}

pub(super) async fn execute<S: CompensationStore>(
    svc: &CompensationServiceImpl<S>,
    req: GetRequest,
) -> Result<GetResponse> {
    let compensation_profile = svc
        .store()
        .get(&req.id)
        .await
        .map_err(map_store_error)?
        .filter(|profile| {
            profile.tenant_id() == &req.tenant_id && profile.employee_id() == &req.employee_id
        });

    Ok(GetResponse {
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
    ) -> CompensationProfile {
        CompensationProfile::new(CompensationProfileDraft {
            tenant_id,
            employee_id,
            amount: CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                .unwrap(),
            cadence: CompensationCadence::Monthly,
            valid_from: None,
            valid_to: None,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn gets_profile_when_it_belongs_to_employee() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let profile = profile_for(tenant_id, employee_id);
        let id = *profile.id();
        let expected = profile.clone();
        let mut store = MockCompensationStore::new();
        store
            .expect_get()
            .withf(move |actual_id| actual_id == &id)
            .returning(move |_| Ok(Some(expected.clone())));

        let svc = CompensationServiceImpl::new(store);
        let response = execute(
            &svc,
            GetRequest {
                tenant_id,
                employee_id,
                id,
            },
        )
        .await
        .unwrap();

        assert_eq!(response.compensation_profile.unwrap().id(), &id);
    }

    #[tokio::test]
    async fn returns_none_when_profile_belongs_to_another_tenant() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let profile = profile_for(StandardID::new(), employee_id);
        let id = *profile.id();
        let mut store = MockCompensationStore::new();
        store
            .expect_get()
            .returning(move |_| Ok(Some(profile.clone())));

        let svc = CompensationServiceImpl::new(store);
        let response = execute(
            &svc,
            GetRequest {
                tenant_id,
                employee_id,
                id,
            },
        )
        .await
        .unwrap();

        assert!(response.compensation_profile.is_none());
    }
}
