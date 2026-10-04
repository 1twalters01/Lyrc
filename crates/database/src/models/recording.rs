use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct RecordingRow {
    pub uuid: Uuid,
    pub song_uuid: Uuid,
    pub artist_uuid: Uuid,
    pub duration_ms: i64,
}
