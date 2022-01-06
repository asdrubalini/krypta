#[derive(Debug, sqlx::FromRow)]
pub struct Tag {
    pub id: u64,
    pub name: String,
}
