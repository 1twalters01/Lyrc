use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct AudioFileRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub file_path: String,
}
