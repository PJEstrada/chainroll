#[derive(Debug, Clone, Copy, Default)]
pub struct Query {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
