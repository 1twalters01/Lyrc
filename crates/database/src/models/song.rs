use sqlx::{Pool, sqlite::Sqlite};
use uuid::Uuid;

#[derive(Debug)]
pub struct SongRow {
    pub uuid: Uuid,
    pub title: String,
}

impl SongRow {
    pub async fn select_by_uuid(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
    ) -> Result<Option<SongRow>, sqlx::Error> {
        sqlx::query_as!(
            SongRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    title
                FROM song
                WHERE uuid = ?
            "#,
            song_uuid,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_title(
        pool: &Pool<Sqlite>,
        song_title: &str,
    ) -> Result<Vec<SongRow>, sqlx::Error> {
        sqlx::query_as!(
            SongRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    title
                FROM song
                WHERE title = ?
            "#,
            song_title,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_artist_uuid(
        pool: &Pool<Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<Vec<SongRow>, sqlx::Error> {
        sqlx::query_as!(
            SongRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    title
                FROM song
                INNER JOIN song_artist
                    ON song.uuid = song_artist.song_uuid
                WHERE song_artist.artist_uuid = ?
            "#,
            artist_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_recording_uuid(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Option<SongRow>, sqlx::Error> {
        sqlx::query_as!(
            SongRow,
            r#"
                SELECT
                    song.uuid AS "uuid: uuid::Uuid",
                    song.title
                FROM song
                INNER JOIN recording
                    ON song.uuid = recording.song_uuid
                WHERE recording.uuid = ?
            "#,
            recording_uuid
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn insert(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO song (uuid, title)
                VALUES (?, ?)
            "#,
            self.uuid,
            self.title,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_title(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
        new_song_title: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE song
                SET title = ?
                WHERE uuid = ?
            "#,
            new_song_title,
            song_uuid,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_title(
        &mut self,
        pool: &Pool<Sqlite>,
        song_title: &str,
    ) -> Result<(), sqlx::Error> {
        Self::update_title(pool, self.uuid, song_title).await?;
        self.title = song_title.to_owned();

        Ok(())
    }

    pub async fn delete(pool: &Pool<Sqlite>, song_uuid: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM song
                WHERE uuid = ?
            "#,
            song_uuid,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(pool: &Pool<Sqlite>, uuid: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song
                    WHERE uuid = ?
                )
            "#,
            uuid,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn title_exists(pool: &Pool<Sqlite>, title: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song
                    WHERE title = ?
                )
            "#,
            title,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
