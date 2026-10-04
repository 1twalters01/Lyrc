#[derive(Debug, sqlx::FromRow)]
pub struct TranslationRow {
    pub id: i64,
    pub name: String,
}
