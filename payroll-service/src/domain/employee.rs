use crate::domain::base_metadata::LifecycleMeta;
use crate::domain::division::IDDivision;
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::query::Query;
use serde_json::Value;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[allow(dead_code)]
pub struct Employee {
    id: StandardID<IDEmployee>,
    metadata: LifecycleMeta,
    identifier: String,
    first_name: String,
    last_name: String,
    divisions: Vec<IDDivision>,
    culture: Option<LanguageIdentifier>,

    // TODO: add calendar relation
    // calendar: Calendar,
    attributes: Option<HashMap<String, Value>>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum EmployeeStatus {
    Active,
    Inactive,
    // TODO: add more as we discover product requirements
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct IDEmployee;

impl IDResource for IDEmployee {
    fn prefix() -> Option<String> {
        Some("employee".to_string())
    }
}

pub struct EmployeeQuery {
    pub base: Query,
    pub division_id: Option<IDDivision>, // employee-specific filter
}
