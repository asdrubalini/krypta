use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub random_hash: String,
    pub data_hash: String,
}

impl File {
    pub async fn search(database: &Database, query: String) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/query_file.sql"))
            .bind(format!("%{}%", query))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}
