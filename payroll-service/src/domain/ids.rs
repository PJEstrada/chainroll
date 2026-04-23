use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;
use std::marker::PhantomData;
use tsid::TSID;

pub trait IDResource: Eq + PartialEq + Clone + Copy + Hash + Send {
    fn prefix() -> Option<String>;
}
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("invalid id format")]
    ParseError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StandardID<Resource: IDResource> {
    pub(crate) id: TSID,
    resource: PhantomData<Resource>,
}
impl<T: IDResource> Default for StandardID<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IDResource> StandardID<T> {
    pub fn new() -> Self {
        Self {
            id: tsid::create_tsid(),
            resource: PhantomData,
        }
    }
}

impl<T: IDResource> std::fmt::Display for StandardID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id) // however TSID displays
    }
}

impl<T: IDResource> std::str::FromStr for StandardID<T> {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tsid = TSID::try_from(s).map_err(|_| IdError::ParseError)?;
        Ok(StandardID {
            id: tsid,
            resource: PhantomData,
        })
    }
}

impl<T: IDResource> Serialize for StandardID<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.id.to_string())
    }
}

impl<'de, T: IDResource> Deserialize<'de> for StandardID<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl<T: IDResource> TryFrom<String> for StandardID<T> {
    type Error = IdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ids::{IDResource, IdError, StandardID};
    use std::str::FromStr;
    #[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
    struct TestID;
    impl IDResource for TestID {
        fn prefix() -> Option<String> {
            None
        }
    }
    #[test]
    fn test_id_from_str() {
        let s = "a wrong id";
        let err = StandardID::<TestID>::from_str(s).unwrap_err();
        assert!(matches!(err, IdError::ParseError));

        let s1 = "000000000003V";
        let val = StandardID::<TestID>::from_str(s1).unwrap();
        assert_eq!(val.id.to_string(), s1);
    }

    #[test]
    fn test_invalid_length() {
        let s = "000000000000000003V";
        let err = StandardID::<TestID>::from_str(s).unwrap_err();
        assert!(matches!(err, IdError::ParseError));
    }

    #[test]
    fn test_can_serialize() {
        let s1 = "000000000003V";
        let val = StandardID::<TestID>::from_str(s1).unwrap();
        let serialized = serde_json::to_string(&val).unwrap();
        let deserialized: StandardID<TestID> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val, deserialized);
    }
}
