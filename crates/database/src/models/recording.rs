use sqlx::{Pool, sqlite::Sqlite};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct RecordingRow {
    pub uuid: Uuid,
    pub song_uuid: Uuid,
    pub duration_ms: i64,
}

impl RecordingRow {
    pub async fn select_by_uuid(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Option<RecordingRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    song_uuid AS "song_uuid: uuid::Uuid",
                    duration_ms
                FROM recording
                WHERE uuid = ?
            "#,
            recording_uuid,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_song_uuid(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
    ) -> Result<Vec<RecordingRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    song_uuid AS "song_uuid: uuid::Uuid",
                    duration_ms
                FROM recording
                WHERE song_uuid = ?
            "#,
            song_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_recording_artist_uuid(
        pool: &Pool<Sqlite>,
        recording_artist_uuid: Uuid,
    ) -> Result<Vec<RecordingRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    song_uuid AS "song_uuid: uuid::Uuid",
                    duration_ms
                FROM recording
                INNER JOIN recording_artist
                    ON recording.uuid = recording_artist.recording_uuid
                WHERE recording_artist.artist_uuid = ?
            "#,
            recording_artist_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_song_artist_uuid(
        pool: &Pool<Sqlite>,
        song_artist_uuid: Uuid,
    ) -> Result<Vec<RecordingRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingRow,
            r#"
                SELECT
                    recording.uuid AS "uuid: uuid::Uuid",
                    recording.song_uuid AS "song_uuid: uuid::Uuid",
                    recording.duration_ms
                FROM recording
                INNER JOIN song_artist
                    ON recording.song_uuid = song_artist.song_uuid
                WHERE song_artist.artist_uuid = ?
            "#,
            song_artist_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn insert(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO recording (uuid, song_uuid, duration_ms)
                VALUES (?, ?, ?)
            "#,
            self.uuid,
            self.song_uuid,
            self.duration_ms,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_duration(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
        duration_ms: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE recording
                SET duration_ms = ?
                WHERE uuid = ?
            "#,
            duration_ms,
            recording_uuid,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_duration(
        &mut self,
        pool: &Pool<Sqlite>,
        duration_ms: i64,
    ) -> Result<(), sqlx::Error> {
        Self::update_duration(pool, self.uuid, duration_ms).await?;
        self.duration_ms = duration_ms;

        Ok(())
    }

    pub async fn delete(pool: &Pool<Sqlite>, recording_uuid: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM recording
                WHERE uuid = ?
            "#,
            recording_uuid,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM recording
                    WHERE uuid = ?
                )
            "#,
            recording_uuid,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
