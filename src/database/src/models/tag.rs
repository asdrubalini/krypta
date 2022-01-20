use database_macros::{FromRow, TableName};

#[derive(TableName, FromRow)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    pub fn new() -> Self {
        Tag {
            name: "urlo-del-sium".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::create_in_memory;

    use super::Tag;
    use crate::traits::TableName;

    #[test]
    fn test_tag() {
        let database = create_in_memory().unwrap();
        let ciao = Tag::new();
        Tag::table_name();
    }
}
