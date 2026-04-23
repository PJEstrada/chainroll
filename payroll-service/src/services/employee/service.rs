use crate::Result;
use crate::services::datastore::EmployeeStore;
use crate::services::employee::count::{CountRequest, CountResponse};
use crate::services::employee::create::{CreateRequest, CreateResponse};
use crate::services::employee::create_batch::{CreateBatchRequest, CreateBatchResponse};
use crate::services::employee::create_many::{CreateManyRequest, CreateManyResponse};
use crate::services::employee::delete::{DeleteRequest, DeleteResponse};
use crate::services::employee::exists::{ExistsRequest, ExistsResponse};
use crate::services::employee::get::{GetRequest, GetResponse};
use crate::services::employee::list::{ListRequest, ListResponse};
use crate::services::employee::update::{UpdateRequest, UpdateResponse};
use crate::services::employee::{
    count, create, create_batch, create_many, delete, exists, get, list, update,
};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait EmployeeService {
    async fn exists(&self, req: ExistsRequest) -> Result<ExistsResponse>;
    async fn get(&self, req: GetRequest) -> Result<GetResponse>;
    async fn count(&self, req: CountRequest) -> Result<CountResponse>;
    async fn list(&self, req: ListRequest) -> Result<ListResponse>;
    async fn create(&self, req: CreateRequest) -> Result<CreateResponse>;
    async fn create_many(&self, req: CreateManyRequest) -> Result<CreateManyResponse>;
    async fn create_batch(&self, req: CreateBatchRequest) -> Result<CreateBatchResponse>;
    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse>;
    async fn delete(&self, req: DeleteRequest) -> Result<DeleteResponse>;
}

#[derive(Debug, Clone)]
pub struct EmployeeServiceImpl<S: EmployeeStore> {
    store: S,
}

impl<S: EmployeeStore> EmployeeServiceImpl<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S: EmployeeStore> EmployeeService for EmployeeServiceImpl<S> {
    async fn exists(&self, req: ExistsRequest) -> Result<ExistsResponse> {
        exists::execute(self, req).await
    }

    async fn get(&self, req: GetRequest) -> Result<GetResponse> {
        get::execute(self, req).await
    }

    async fn count(&self, req: CountRequest) -> Result<CountResponse> {
        count::execute(self, req).await
    }

    async fn list(&self, req: ListRequest) -> Result<ListResponse> {
        list::execute(self, req).await
    }

    async fn create(&self, req: CreateRequest) -> Result<CreateResponse> {
        create::execute(self, req).await
    }

    async fn create_many(&self, req: CreateManyRequest) -> Result<CreateManyResponse> {
        create_many::execute(self, req).await
    }

    async fn create_batch(&self, req: CreateBatchRequest) -> Result<CreateBatchResponse> {
        create_batch::execute(self, req).await
    }

    async fn update(&self, req: UpdateRequest) -> Result<UpdateResponse> {
        update::execute(self, req).await
    }

    async fn delete(&self, req: DeleteRequest) -> Result<DeleteResponse> {
        delete::execute(self, req).await
    }
}
