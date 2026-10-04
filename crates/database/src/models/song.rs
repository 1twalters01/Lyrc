use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct SongRow {
    pub uuid: Uuid,
    pub title: String,
    pub artist_uuid: Uuid,
}
