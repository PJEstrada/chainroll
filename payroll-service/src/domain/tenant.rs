use crate::domain::ids::IDResource;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDTenant;

impl IDResource for IDTenant {
    fn prefix() -> Option<String> {
        Some("tenant".to_string())
    }
}
