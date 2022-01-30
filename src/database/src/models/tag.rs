use std::fmt::Display;

use database_macros::{Insert, TableName, TryFromRow};

#[derive(TableName, TryFromRow, Insert)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Tag {
    pub fn new(name: impl AsRef<str>) -> Self {
        Tag {
            id: None,
            name: name.as_ref().to_string(),
        }
    }
}
