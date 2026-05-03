use crate::error::Error;
use crate::services::datastore::TreasuryStore;
use crate::services::datastore::postgres::treasury_store::TreasuryStoreError;
use crate::services::treasury::create::{CreateRequest, CreateResponse};
use crate::services::treasury::deactivate::{DeactivateRequest, DeactivateResponse};
use crate::services::treasury::get::{GetRequest, GetResponse};
use crate::services::treasury::list::{ListRequest, ListResponse};
use crate::services::treasury::update::{UpdateRequest, UpdateResponse};
use crate::services::treasury::{create, deactivate, get, list, update};
use crate::{Result, error};
use error_stack::Report;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait TreasuryService {
    async fn get(&self, req: GetRequest) -> Result<GetResponse>;
    async fn list(&self, req: ListRequest) -> Result<ListResponse>;
    async fn create(&self, req: CreateRequest) -> Result<CreateResponse>;
    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse>;
    async fn deactivate(&self, req: DeactivateRequest) -> Result<DeactivateResponse>;
}

#[derive(Debug, Clone)]
pub struct TreasuryServiceImpl<S: TreasuryStore> {
    store: S,
}

impl<S: TreasuryStore> TreasuryServiceImpl<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S: TreasuryStore> TreasuryService for TreasuryServiceImpl<S> {
    async fn get(&self, req: GetRequest) -> Result<GetResponse> {
        get::execute(self, req).await
    }

    async fn list(&self, req: ListRequest) -> Result<ListResponse> {
        list::execute(self, req).await
    }

    async fn create(&self, req: CreateRequest) -> Result<CreateResponse> {
        create::execute(self, req).await
    }

    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse> {
        update::execute(self, req).await
    }

    async fn deactivate(&self, req: DeactivateRequest) -> Result<DeactivateResponse> {
        deactivate::execute(self, req).await
    }
}

pub(super) fn map_store_error(err: TreasuryStoreError) -> Report<Error> {
    match err {
        TreasuryStoreError::TreasuryAccountNotFound => Report::new(error::Error::NotFound),
        TreasuryStoreError::InvalidId(_)
        | TreasuryStoreError::InvalidStatus(_)
        | TreasuryStoreError::InvalidChain(_)
        | TreasuryStoreError::InvalidCustodyProvider(_)
        | TreasuryStoreError::InvalidControlMode(_)
        | TreasuryStoreError::InvalidTreasuryAccount(_)
        | TreasuryStoreError::InvalidWalletAddress(_)
        | TreasuryStoreError::InvalidChainId { .. }
        | TreasuryStoreError::InvalidTokenDecimals { .. } => {
            Report::new(error::Error::InvalidInput(err.to_string()))
        }
        TreasuryStoreError::Database(_)
        | TreasuryStoreError::Audit(_)
        | TreasuryStoreError::UnexpectedError(_) => {
            Report::new(err).change_context(error::Error::Database)
        }
    }
}
