use crate::domain::ids::IDResource;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDUser;

impl IDResource for IDUser {
    fn prefix() -> Option<String> {
        Some("user".to_string())
    }
}
