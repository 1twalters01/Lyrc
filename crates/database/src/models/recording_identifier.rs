use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct RecordingIdentifierRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub source_id: i64,
    pub identifier: String,
}
