use sqlx::{Pool, sqlite::Sqlite};
use uuid::Uuid;

#[derive(Debug)]
pub struct SongArtistRow {
    pub song_uuid: Uuid,
    pub artist_uuid: Uuid,
}

impl SongArtistRow {
    pub async fn select(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<Option<SongArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            SongArtistRow,
            r#"
                SELECT
                    song_uuid AS "song_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM song_artist
                WHERE song_uuid = ?
                  AND artist_uuid = ?
            "#,
            song_uuid,
            artist_uuid,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_song_uuid(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
    ) -> Result<Vec<SongArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            SongArtistRow,
            r#"
                SELECT
                    song_uuid AS "song_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM song_artist
                WHERE song_uuid = ?
            "#,
            song_uuid,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_artist_uuid(
        pool: &Pool<Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<Vec<SongArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            SongArtistRow,
            r#"
                SELECT
                    song_uuid AS "song_uuid: uuid::Uuid",
                    artist_uuid AS "artist_uuid: uuid::Uuid"
                FROM song_artist
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
                INSERT INTO song_artist (song_uuid, artist_uuid)
                VALUES (?, ?)
            "#,
            self.song_uuid,
            self.artist_uuid,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM song_artist
                WHERE song_uuid = ?
                  AND artist_uuid = ?
            "#,
            song_uuid,
            artist_uuid,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song_artist
                    WHERE song_uuid = ?
                      AND artist_uuid = ?
                )
            "#,
            song_uuid,
            artist_uuid,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
