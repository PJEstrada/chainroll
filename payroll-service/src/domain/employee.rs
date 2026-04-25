use crate::domain::base_metadata::LifecycleMeta;
use crate::domain::division::IDDivision;
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::query::Query;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::DisplayFromStr;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    id: StandardID<IDEmployee>,
    metadata: LifecycleMeta,
    identifier: String,
    first_name: String,
    last_name: String,
    divisions: Vec<StandardID<IDDivision>>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    culture: Option<LanguageIdentifier>,

    // TODO: add calendar relation
    // calendar: Calendar,
    attributes: Option<HashMap<String, Value>>,
}

impl Employee {
    pub fn new(identifier: String, first_name: String, last_name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: StandardID::new(),
            metadata: LifecycleMeta {
                status: crate::domain::base_metadata::ObjectStatus::Active,
                created: now,
                updated: now,
            },
            identifier,
            first_name,
            last_name,
            divisions: Vec::new(),
            culture: None,
            attributes: None,
        }
    }

    pub fn with_id(mut self, id: StandardID<IDEmployee>) -> Self {
        self.id = id;
        self
    }
    pub fn with_divisions(mut self, divisions: Vec<StandardID<IDDivision>>) -> Self {
        self.divisions = divisions;
        self
    }

    pub fn with_culture(mut self, culture: Option<LanguageIdentifier>) -> Self {
        self.culture = culture;
        self
    }

    pub fn with_metadata(mut self, metadata: LifecycleMeta) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_attributes(mut self, attributes: Option<HashMap<String, Value>>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn id(&self) -> &StandardID<IDEmployee> {
        &self.id
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }
    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn divisions(&self) -> &Vec<StandardID<IDDivision>> {
        &self.divisions
    }
    pub fn culture(&self) -> &Option<LanguageIdentifier> {
        &self.culture
    }
    pub fn attributes(&self) -> &Option<HashMap<String, Value>> {
        &self.attributes
    }
    pub fn metadata(&self) -> &LifecycleMeta {
        &self.metadata
    }
    pub fn status(&self) -> &crate::domain::base_metadata::ObjectStatus {
        &self.metadata.status
    }
    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.metadata.created
    }
    pub fn updated_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.metadata.updated
    }
    pub fn set_status(&mut self, status: crate::domain::base_metadata::ObjectStatus) {
        self.metadata.status = status;
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum EmployeeStatus {
    Active,
    Inactive,
    // TODO: add more as we discover product requirements
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDEmployee;

impl IDResource for IDEmployee {
    fn prefix() -> Option<String> {
        Some("employee".to_string())
    }
}

pub struct EmployeeQuery {
    pub base: Query,
    pub division_id: Option<StandardID<IDDivision>>, // employee-specific filter
}
