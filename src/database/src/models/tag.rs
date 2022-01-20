use database_macros::TableName;

use crate::traits::TableName;

#[derive(TableName, Debug)]
pub struct Tag {
    pub name: String,
}
