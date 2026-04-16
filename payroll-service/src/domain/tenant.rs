use crate::domain::ids::IDResource;

pub struct Tenant {
    pub id: IDTenant,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct IDTenant;

impl IDResource for IDTenant {
    fn prefix() -> Option<String> {
        Some("tenant".to_string())
    }
}
