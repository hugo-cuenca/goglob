use crate::GlobPattern;
use serde::{de::Error, Deserialize, Deserializer};

impl<'de> Deserialize<'de> for GlobPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        GlobPattern::new(string).map_err(D::Error::custom)
    }
}
