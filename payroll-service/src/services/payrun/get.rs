use crate::Result;
use crate::domain::ids::StandardID;
use crate::domain::payrun::{IDPayrun, Payrun};
use crate::domain::tenant::IDTenant;
use crate::services::datastore::{CompensationStore, EmployeeStore, PayrunStore, TreasuryStore};
use crate::services::payrun::service::{PayrunServiceImpl, map_store_error};

pub struct GetRequest {
    pub tenant_id: StandardID<IDTenant>,
    pub id: StandardID<IDPayrun>,
}

pub struct GetResponse {
    pub payrun: Option<Payrun>,
}

pub(super) async fn execute<
    E: EmployeeStore,
    C: CompensationStore,
    T: TreasuryStore,
    P: PayrunStore,
>(
    svc: &PayrunServiceImpl<E, C, T, P>,
    req: GetRequest,
) -> Result<GetResponse> {
    let payrun = svc
        .payrun_store()
        .get(&req.tenant_id, &req.id)
        .await
        .map_err(map_store_error)?;

    Ok(GetResponse { payrun })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::compensation::CompensationAmount;
    use crate::domain::employee::IDEmployee;
    use crate::domain::payrun::{CreatePayrunOptions, PayrunPreview, PayrunPreviewItem};
    use crate::domain::treasury::TokenSymbol;
    use crate::services::datastore::{
        MockCompensationStore, MockEmployeeStore, MockPayrunStore, MockTreasuryStore,
    };

    fn payrun() -> Payrun {
        Payrun::new(
            PayrunPreview::new(
                StandardID::new(),
                vec![PayrunPreviewItem::payable(
                    StandardID::<IDEmployee>::new(),
                    CompensationAmount::new(1_000_000, TokenSymbol::parse("USDC").unwrap())
                        .unwrap(),
                )],
            ),
            CreatePayrunOptions::strict(),
        )
        .unwrap()
    }

    fn service(
        payrun: Option<Payrun>,
    ) -> PayrunServiceImpl<
        MockEmployeeStore,
        MockCompensationStore,
        MockTreasuryStore,
        MockPayrunStore,
    > {
        let mut payrun_store = MockPayrunStore::new();
        payrun_store
            .expect_get()
            .returning(move |_, _| Ok(payrun.clone()));

        PayrunServiceImpl::new(
            MockEmployeeStore::new(),
            MockCompensationStore::new(),
            MockTreasuryStore::new(),
            payrun_store,
        )
    }

    #[tokio::test]
    async fn gets_payrun_by_tenant_and_id() {
        let payrun = payrun();
        let id = *payrun.id();
        let tenant_id = *payrun.tenant_id();
        let svc = service(Some(payrun));

        let response = execute(&svc, GetRequest { tenant_id, id }).await.unwrap();

        assert_eq!(response.payrun.unwrap().id(), &id);
    }

    #[tokio::test]
    async fn returns_none_when_payrun_is_missing() {
        let svc = service(None);

        let response = execute(
            &svc,
            GetRequest {
                tenant_id: StandardID::new(),
                id: StandardID::new(),
            },
        )
        .await
        .unwrap();

        assert!(response.payrun.is_none());
    }
}
