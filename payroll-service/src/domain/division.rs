use crate::domain::ids::{IDResource, StandardID};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub struct Division {
    id: StandardID<IDDivision>,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDDivision;

impl IDResource for IDDivision {
    fn prefix() -> Option<String> {
        Some("division".to_string())
    }
}
