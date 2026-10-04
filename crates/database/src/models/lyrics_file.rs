use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct LyricsFileRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub variant_id: i64,
    pub format_id: i64,
    pub file_path: String,
}
