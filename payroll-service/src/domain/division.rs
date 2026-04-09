use crate::domain::ids::{IDResource, StandardID};

#[allow(dead_code)]
pub struct Division {
    id: StandardID<IDDivision>,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct IDDivision;

impl IDResource for IDDivision {
    fn prefix() -> Option<String> {
        Some("division".to_string())
    }
}
