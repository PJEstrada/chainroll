use crate::Result;
use crate::services::datastore::{CompensationStore, EmployeeStore, PayrunStore, TreasuryStore};
use crate::services::payrun::preview;
use crate::services::payrun::preview::{PreviewRequest, PreviewResponse};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayrunService {
    async fn preview(&self, req: PreviewRequest) -> Result<PreviewResponse>;
}
#[derive(Debug, Clone)]
pub struct PayrunServiceImpl<E, C, T, P>
where
    E: EmployeeStore,
    C: CompensationStore,
    T: TreasuryStore,
    P: PayrunStore,
{
    employee_store: E,
    compensation_store: C,
    treasury_store: T,
    payrun_store: P,
}

impl<E: EmployeeStore, C: CompensationStore, T: TreasuryStore, P: PayrunStore>
    PayrunServiceImpl<E, C, T, P>
{
    pub fn new(
        employee_store: E,
        compensation_store: C,
        treasury_store: T,
        payrun_store: P,
    ) -> Self {
        Self {
            employee_store,
            compensation_store,
            treasury_store,
            payrun_store,
        }
    }
    pub fn employee_store(&self) -> &E {
        &self.employee_store
    }
    pub fn compensation_store(&self) -> &C {
        &self.compensation_store
    }
    pub fn treasury_store(&self) -> &T {
        &self.treasury_store
    }
    pub fn payrun_store(&self) -> &P {
        &self.payrun_store
    }
}

impl<E: EmployeeStore, C: CompensationStore, T: TreasuryStore, P: PayrunStore> PayrunService
    for PayrunServiceImpl<E, C, T, P>
{
    async fn preview(&self, req: PreviewRequest) -> Result<PreviewResponse> {
        preview::execute(self, req).await
    }
}
