use std::hash::Hash;
use std::marker::PhantomData;
use tsid::TSID;

pub trait IDResource: Eq + PartialEq + Clone + Copy + Hash + Send {
    fn prefix() -> Option<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StandardID<Resource: IDResource> {
    pub(crate) id: TSID,
    resource: PhantomData<Resource>,
}
