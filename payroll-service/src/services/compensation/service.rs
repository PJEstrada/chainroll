use crate::error::Error;
use crate::services::compensation::create::{CreateRequest, CreateResponse};
use crate::services::compensation::get::{GetRequest, GetResponse};
use crate::services::compensation::get_active_for_employee::{
    GetActiveForEmployeeRequest, GetActiveForEmployeeResponse,
};
use crate::services::compensation::list_for_employee::{
    ListForEmployeeRequest, ListForEmployeeResponse,
};
use crate::services::compensation::update::{UpdateRequest, UpdateResponse};
use crate::services::compensation::{
    create, get, get_active_for_employee, list_for_employee, update,
};
use crate::services::datastore::CompensationStore;
use crate::services::datastore::postgres::compensation_store::CompensationStoreError;
use crate::{Result, error};
use error_stack::Report;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait CompensationService: Send + Sync + 'static {
    async fn create(&self, req: CreateRequest) -> Result<CreateResponse>;
    async fn get(&self, req: GetRequest) -> Result<GetResponse>;
    async fn get_active_for_employee(
        &self,
        req: GetActiveForEmployeeRequest,
    ) -> Result<GetActiveForEmployeeResponse>;
    async fn list_for_employee(
        &self,
        req: ListForEmployeeRequest,
    ) -> Result<ListForEmployeeResponse>;
    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse>;
}

#[derive(Debug, Clone)]
pub struct CompensationServiceImpl<S: CompensationStore> {
    store: S,
}

impl<S: CompensationStore> CompensationServiceImpl<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S: CompensationStore> CompensationService for CompensationServiceImpl<S> {
    async fn create(&self, req: CreateRequest) -> Result<CreateResponse> {
        create::execute(self, req).await
    }

    async fn get(&self, req: GetRequest) -> Result<GetResponse> {
        get::execute(self, req).await
    }

    async fn get_active_for_employee(
        &self,
        req: GetActiveForEmployeeRequest,
    ) -> Result<GetActiveForEmployeeResponse> {
        get_active_for_employee::execute(self, req).await
    }

    async fn list_for_employee(
        &self,
        req: ListForEmployeeRequest,
    ) -> Result<ListForEmployeeResponse> {
        list_for_employee::execute(self, req).await
    }

    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse> {
        update::execute(self, req).await
    }
}

pub(super) fn map_store_error(err: CompensationStoreError) -> Report<Error> {
    if is_one_active_profile_conflict(&err) {
        return Report::new(error::Error::Conflict(
            "active compensation profile already exists for employee".to_string(),
        ));
    }

    match err {
        CompensationStoreError::CompensationProfileNotFound => Report::new(error::Error::NotFound),
        CompensationStoreError::CompensationAlreadyExists => Report::new(error::Error::Conflict(
            "compensation profile already exists".to_string(),
        )),
        CompensationStoreError::InvalidId(_)
        | CompensationStoreError::InvalidStatus(_)
        | CompensationStoreError::InvalidTokenSymbol(_)
        | CompensationStoreError::InvalidCompensationProfile(_)
        | CompensationStoreError::InvalidAmountUnits
        | CompensationStoreError::InvalidCadenceEvery => {
            Report::new(error::Error::InvalidInput(err.to_string()))
        }
        CompensationStoreError::Database(_) | CompensationStoreError::Audit(_) => {
            Report::new(err).change_context(error::Error::Database)
        }
    }
}

fn is_one_active_profile_conflict(err: &CompensationStoreError) -> bool {
    matches!(
        err,
        CompensationStoreError::Database(sqlx::Error::Database(db_error))
            if db_error.constraint() == Some("compensation_profiles_one_active_per_employee_idx")
    )
}
