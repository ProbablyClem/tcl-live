use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_tcl_date(&s).map_err(serde::de::Error::custom)
}

pub fn parse_tcl_date(date: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
}
