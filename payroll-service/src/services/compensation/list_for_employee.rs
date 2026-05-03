use crate::Result;
use crate::domain::compensation::CompensationProfile;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::services::compensation::service::{CompensationServiceImpl, map_store_error};
use crate::services::datastore::CompensationStore;

pub struct ListForEmployeeRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub employee_id: StandardID<IDEmployee>,
}

pub struct ListForEmployeeResponse {
    pub compensation_profiles: Vec<CompensationProfile>,
}

pub(super) async fn execute<S: CompensationStore>(
    svc: &CompensationServiceImpl<S>,
    req: ListForEmployeeRequest,
) -> Result<ListForEmployeeResponse> {
    let compensation_profiles = svc
        .store()
        .list_for_employee(&req.tenant_id, &req.employee_id)
        .await
        .map_err(map_store_error)?;

    Ok(ListForEmployeeResponse {
        compensation_profiles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::datastore::MockCompensationStore;

    #[tokio::test]
    async fn calls_store_with_tenant_and_employee() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let mut store = MockCompensationStore::new();
        store
            .expect_list_for_employee()
            .withf(move |actual_tenant_id, actual_employee_id| {
                actual_tenant_id == &tenant_id && actual_employee_id == &employee_id
            })
            .returning(|_, _| Ok(Vec::new()));

        let svc = CompensationServiceImpl::new(store);
        let response = execute(
            &svc,
            ListForEmployeeRequest {
                tenant_id,
                employee_id,
            },
        )
        .await
        .unwrap();

        assert!(response.compensation_profiles.is_empty());
    }
}
