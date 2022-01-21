use database_macros::TryFromRow;

#[derive(TryFromRow)]
pub struct FileSearch {
    pub path: String,
}
