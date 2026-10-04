#[derive(Debug, sqlx::FromRow)]
pub struct SourceRow {
    pub id: i64,
    pub source: String,
}
