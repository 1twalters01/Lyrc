use sqlx::{Pool, sqlite::Sqlite};
use uuid::Uuid;

#[derive(Debug)]
pub struct ArtistRow {
    pub uuid: Uuid,
    pub name: String,
}

impl ArtistRow {
    pub async fn select_by_uuid(
        pool: &Pool<Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<Option<ArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            ArtistRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    name
                FROM artist
                WHERE uuid = ?
            "#,
            artist_uuid,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_name(
        pool: &Pool<Sqlite>,
        artist_name: &str,
    ) -> Result<Vec<ArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            ArtistRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    name
                FROM artist
                WHERE name = ?
            "#,
            artist_name,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_song_uuid(
        pool: &Pool<Sqlite>,
        song_uuid: Uuid,
    ) -> Result<Vec<ArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            ArtistRow,
            r#"
                SELECT
                    artist.uuid AS "uuid: uuid::Uuid",
                    artist.name
                FROM artist
                INNER JOIN song_artist
                    ON artist.uuid = song_artist.artist_uuid
                WHERE song_artist.song_uuid = ?
            "#,
            song_uuid
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_recording_uuid(
        pool: &Pool<Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Vec<ArtistRow>, sqlx::Error> {
        sqlx::query_as!(
            ArtistRow,
            r#"
                SELECT
                    artist.uuid AS "uuid: uuid::Uuid",
                    artist.name
                FROM artist
                INNER JOIN recording_artist
                    ON artist.uuid = recording_artist.artist_uuid
                WHERE recording_artist.recording_uuid = ?
            "#,
            recording_uuid
        )
        .fetch_all(pool)
        .await
    }

    pub async fn insert(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO artist (uuid, name)
                VALUES (?, ?)
            "#,
            self.uuid,
            self.name,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_name(
        pool: &Pool<Sqlite>,
        artist_uuid: Uuid,
        new_artist_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE artist
                SET name = ?
                WHERE uuid = ?
            "#,
            new_artist_name,
            artist_uuid,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_name(
        &mut self,
        pool: &Pool<Sqlite>,
        artist_name: &str,
    ) -> Result<(), sqlx::Error> {
        Self::update_name(pool, self.uuid, &artist_name).await?;
        self.name = artist_name.to_owned();

        Ok(())
    }

    pub async fn delete(pool: &Pool<Sqlite>, artist_uuid: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                Delete FROM artist
                WHERE uuid = ?
            "#,
            artist_uuid,
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
                FROM artist
                WHERE uuid = ?
            )
        "#,
            uuid,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn name_exists(pool: &Pool<Sqlite>, name: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM artist
                WHERE name = ?
            )
        "#,
            name,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
