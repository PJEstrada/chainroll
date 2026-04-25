use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy, Hash, Deserialize)]
pub enum ObjectStatus {
    Active,
    Inactive,
}
impl Display for ObjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectStatus::Active => write!(f, "active"),
            ObjectStatus::Inactive => write!(f, "inactive"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid status: {0}")]
pub struct ParseStatusError(String);

impl FromStr for ObjectStatus {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" | "Active" => Ok(Self::Active),
            "inactive" | "Inactive" => Ok(Self::Inactive),
            other => Err(ParseStatusError(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMeta {
    pub status: ObjectStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
