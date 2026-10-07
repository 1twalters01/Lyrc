use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ArtistRow {
    pub uuid: Uuid,
    pub name: String,
}

impl ArtistRow {
    pub fn from_name(artist_name: &str) -> Self {
        ArtistRow {
            uuid: Uuid::new_v4(),
            name: artist_name.to_string(),
        }
    }

    pub async fn select_by_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_name(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_song_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_recording_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn insert(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO artist (uuid, name)
                VALUES (?, ?)
            "#,
            self.uuid,
            self.name,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_or_create_by_names(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        names: &[String],
    ) -> Result<Vec<ArtistRow>, sqlx::Error> {
        let mut artists = Vec::with_capacity(names.len());

        for name in names {
            let artist = match Self::select_by_name(tx, name).await?.first() {
                Some(artist) => artist.clone(),
                None => {
                    let artist = ArtistRow::from_name(name);
                    artist.insert(tx).await?;
                    artist
                }
            };

            artists.push(artist);
        }

        Ok(artists)
    }

    pub async fn update_name(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn set_name(
        &mut self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        artist_name: &str,
    ) -> Result<(), sqlx::Error> {
        Self::update_name(tx, self.uuid, &artist_name).await?;
        self.name = artist_name.to_owned();

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                Delete FROM artist
                WHERE uuid = ?
            "#,
            artist_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        artist_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM artist
                WHERE uuid = ?
            )
        "#,
            artist_uuid,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn name_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        artist_name: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM artist
                WHERE name = ?
            )
        "#,
            artist_name,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
