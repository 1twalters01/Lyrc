#[derive(Debug, sqlx::FromRow)]
pub struct LyricsFormatRow {
    pub id: i64,
    pub format: String,
}
