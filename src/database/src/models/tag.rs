#[derive(Debug, sqlx::FromRow)]
pub struct Tag {
    pub name: String,
}
