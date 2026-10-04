use sqlx::{Pool, sqlite::Sqlite};
use uuid::Uuid;

#[derive(Debug)]
pub struct RecordingArtistRow {
    pub recording_uuid: Uuid,
    pub artist_uuid: Uuid,
}

impl RecordingArtistRow {
    pub async fn select_by_uuids(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<Option<RecordingArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingArtistRow,
            r#"
                SELECT
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM recording_artist
                WHERE recording_uuid = ?
                  AND artist_uuid = ?
            "#,
            recording_uuid,
            artist_uuid,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_recording_uuid(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Vec<RecordingArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingArtistRow,
            r#"
                SELECT
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM recording_artist
                WHERE recording_uuid = ?
            "#,
            recording_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_artist_uuid(
        pool: &Pool<Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<Vec<RecordingArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingArtistRow,
            r#"
                SELECT
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM recording_artist
                WHERE artist_uuid = ?
            "#,
            artist_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn insert(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO recording_artist (recording_uuid, artist_uuid)
                VALUES (?, ?)
            "#,
            self.recording_uuid,
            self.artist_uuid,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM recording_artist
                WHERE recording_uuid = ?
                  AND artist_uuid = ?
            "#,
            recording_uuid,
            artist_uuid,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM recording_artist
                    WHERE recording_uuid = ?
                      AND artist_uuid = ?
                )
            "#,
            recording_uuid,
            artist_uuid,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
