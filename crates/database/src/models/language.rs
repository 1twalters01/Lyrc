#[derive(Debug, sqlx::FromRow)]
pub struct LanguageRow {
    pub id: i64,
    pub code_3: String,
}
