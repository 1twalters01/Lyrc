use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SongRow {
    pub uuid: Uuid,
    pub title: String,
}

impl SongRow {
    pub fn from_title(song_title: &str) -> Self {
        SongRow {
            uuid: Uuid::new_v4(),
            title: song_title.to_string(),
        }
    }

    pub async fn select_by_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_title(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_artist_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_recording_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn insert(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO song (uuid, title)
                VALUES (?, ?)
            "#,
            self.uuid,
            self.title,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_or_create_by_titles(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        titles: &[String],
    ) -> Result<Vec<SongRow>, sqlx::Error> {
        let mut songs = Vec::with_capacity(titles.len());

        for title in titles {
            let song = match Self::select_by_title(tx, title).await?.first() {
                Some(song) => song.clone(),
                None => {
                    let song = SongRow {
                        uuid: Uuid::new_v4(),
                        title: title.to_string(),
                    };

                    song.insert(tx).await?;
                    song
                }
            };

            songs.push(song);
        }

        Ok(songs)
    }

    pub async fn update_title(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn set_title(
        &mut self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        song_title: &str,
    ) -> Result<(), sqlx::Error> {
        Self::update_title(tx, self.uuid, song_title).await?;
        self.title = song_title.to_owned();

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        song_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM song
                WHERE uuid = ?
            "#,
            song_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        song_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song
                    WHERE uuid = ?
                )
            "#,
            song_uuid,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn title_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        song_title: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM song
                    WHERE title = ?
                )
            "#,
            song_title,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
