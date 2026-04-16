use std::future::Future;

use crate::Result;
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

pub trait EmployeeService {
    fn exists(&self, req: ExistsRequest) -> impl Future<Output = Result<ExistsResponse>> + Send;
    fn get(&self, req: GetRequest) -> impl Future<Output = Result<GetResponse>> + Send;
    fn count(&self, req: CountRequest) -> impl Future<Output = Result<CountResponse>> + Send;
    fn list(&self, req: ListRequest) -> impl Future<Output = Result<ListResponse>> + Send;
    fn create(&self, req: CreateRequest) -> impl Future<Output = Result<CreateResponse>> + Send;
    fn create_many(
        &self,
        req: CreateManyRequest,
    ) -> impl Future<Output = Result<CreateManyResponse>> + Send;
    fn create_batch(
        &self,
        req: CreateBatchRequest,
    ) -> impl Future<Output = Result<CreateBatchResponse>> + Send;
    fn update(&self, req: UpdateRequest) -> impl Future<Output = Result<UpdateResponse>> + Send;
    fn delete(&self, req: DeleteRequest) -> impl Future<Output = Result<DeleteResponse>> + Send;
}

#[derive(Debug, Clone)]
pub struct EmployeeServiceImpl;

impl EmployeeService for EmployeeServiceImpl {
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
