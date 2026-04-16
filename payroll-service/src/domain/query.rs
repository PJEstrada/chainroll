use crate::domain::base_metadata::ObjectStatus;

#[derive(Debug, Default)]
pub struct Query {
    // pagination
    pub limit: Option<u32>,
    pub offset: Option<u32>,

    // ordering
    pub order_by: Option<OrderBy>,

    // filtering
    pub status: Option<ObjectStatus>,
}

#[derive(Debug)]
pub struct OrderBy {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}
