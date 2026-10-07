use uuid::Uuid;

#[derive(Debug)]
pub struct SongArtistRow {
    pub song_uuid: Uuid,
    pub artist_uuid: Uuid,
}

impl SongArtistRow {
    pub async fn select_by_uuids(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_song_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_artist_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn insert(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO song_artist (song_uuid, artist_uuid)
                VALUES (?, ?)
            "#,
            self.song_uuid,
            self.artist_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn exists_by_name(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        song_title: &str,
        artist_name: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song_artist
                    JOIN song
                        ON song.uuid = song_artist.song_uuid
                    JOIN artist
                        ON artist.uuid = song_artist.artist_uuid
                    WHERE song.title = ?
                        AND artist.name = ?
                )
            "#,
            song_title,
            artist_name,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
