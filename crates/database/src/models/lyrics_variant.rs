#[derive(Debug, sqlx::FromRow)]
pub struct LyricsVariantRow {
    pub id: i64,
    pub translation_id: i64,
    pub language_id: i64,
}
