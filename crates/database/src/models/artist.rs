use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ArtistRow {
    pub uuid: Uuid,
    pub name: String,
}
