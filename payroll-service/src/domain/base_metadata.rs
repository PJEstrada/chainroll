use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum ObjectStatus {
    Active,
    Inactive,
}

pub struct LifecycleMeta {
    pub status: ObjectStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
