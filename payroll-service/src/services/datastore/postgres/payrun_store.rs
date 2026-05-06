use crate::services::datastore::PayrunStore;
use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayrunStoreError {
    #[error("Compensation profile not found")]
    CompensationProfileNotFound,
}

#[derive(Debug, Clone)]
pub struct PgPayrunStore {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PgPayrunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PayrunStore for PgPayrunStore {}
