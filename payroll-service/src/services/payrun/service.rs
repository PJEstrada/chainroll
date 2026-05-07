use crate::Result;
use crate::error::Error;
use crate::services::datastore::postgres::payrun_store::PayrunStoreError;
use crate::services::datastore::{CompensationStore, EmployeeStore, PayrunStore, TreasuryStore};
use crate::services::payrun::create;
use crate::services::payrun::create::{CreateRequest, CreateResponse};
use crate::services::payrun::get;
use crate::services::payrun::get::{GetRequest, GetResponse};
use crate::services::payrun::preview;
use crate::services::payrun::preview::{PreviewRequest, PreviewResponse};
use error_stack::Report;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayrunService {
    async fn preview(&self, req: PreviewRequest) -> Result<PreviewResponse>;
    async fn create(&self, req: CreateRequest) -> Result<CreateResponse>;
    async fn get(&self, req: GetRequest) -> Result<GetResponse>;
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

    async fn create(&self, req: CreateRequest) -> Result<CreateResponse> {
        create::execute(self, req).await
    }

    async fn get(&self, req: GetRequest) -> Result<GetResponse> {
        get::execute(self, req).await
    }
}

pub(super) fn map_store_error(err: PayrunStoreError) -> Report<Error> {
    match err {
        PayrunStoreError::InvalidId(_)
        | PayrunStoreError::InvalidPayrunStatus(_)
        | PayrunStoreError::InvalidPayrunItemStatus(_)
        | PayrunStoreError::InvalidTokenSymbol(_)
        | PayrunStoreError::InvalidAmountUnits
        | PayrunStoreError::InvalidPayrun(_) => Report::new(Error::InvalidInput(err.to_string())),
        PayrunStoreError::Database(_) | PayrunStoreError::Audit(_) => {
            Report::new(err).change_context(Error::Database)
        }
    }
}
