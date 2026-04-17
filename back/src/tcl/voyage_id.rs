use fixture_rs::Fixture;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VoyageId {
    id: String,
}

impl VoyageId {
    pub fn value(&self) -> &str {
        &self.id
    }
}

impl<'de> Deserialize<'de> for VoyageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(VoyageId { id: s })
    }
}

impl Serialize for VoyageId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.id)
    }
}

impl std::hash::Hash for VoyageId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Fixture for VoyageId {
    fn fixture() -> Self {
        VoyageId {
            id: "31_31B-023AT_00601030".to_string(),
        }
    }
}
