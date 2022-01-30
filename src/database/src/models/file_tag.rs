use database_macros::{Insert, TableName, TryFromRow};

use super::{File, Tag};

#[derive(TableName, TryFromRow, Insert)]
pub struct FileTag {
    file_id: i64,
    tag_id: i64,
}

impl FileTag {
    pub fn new(file: &File, tag: &Tag) -> Self {
        FileTag {
            file_id: file.id.expect("missing file.id"),
            tag_id: tag.id.expect("missing tag.id"),
        }
    }
}
