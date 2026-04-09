use chrono::{DateTime, Utc};

pub enum ObjectStatus {
    Active,
    Inactive,
}

pub struct AuditInfo {
    pub status: ObjectStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
