use uuid::Uuid;

#[derive(Debug)]
pub struct LyricsFileRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub variant_id: i64,
    pub format_id: i64,
    pub file_path: String,
}

impl LyricsFileRow {
    pub async fn select_by_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        lyrics_file_uuid: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    variant_id,
                    format_id,
                    file_path
                FROM lyrics_file
                WHERE uuid = ?
            "#,
            lyrics_file_uuid,
        )
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_recording_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    variant_id,
                    format_id,
                    file_path
                FROM lyrics_file
                WHERE recording_uuid = ?
            "#,
            recording_uuid,
        )
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_variant_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        variant_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    variant_id,
                    format_id,
                    file_path
                FROM lyrics_file
                WHERE variant_id = ?
            "#,
            variant_id,
        )
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_file_path(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        file_path: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    variant_id,
                    format_id,
                    file_path
                FROM lyrics_file
                WHERE file_path = ?
            "#,
            file_path,
        )
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_uuid: Uuid,
        variant_id: i64,
        format_id: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    variant_id,
                    format_id,
                    file_path
                FROM lyrics_file
                WHERE recording_uuid = ?
                  AND variant_id = ?
                  AND format_id = ?
            "#,
            recording_uuid,
            variant_id,
            format_id,
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
                INSERT INTO lyrics_file (
                    uuid,
                    recording_uuid,
                    variant_id,
                    format_id,
                    file_path
                )
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(recording_uuid, variant_id, format_id)
                DO UPDATE SET
                    file_path = excluded.file_path
            "#,
            self.uuid,
            self.recording_uuid,
            self.variant_id,
            self.format_id,
            self.file_path,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        lyrics_file_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM lyrics_file
                WHERE uuid = ?
            "#,
            lyrics_file_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_file_path(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        lyrics_file_uuid: Uuid,
        file_path: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                UPDATE lyrics_file
                SET file_path = ?
                WHERE uuid = ?
            "#,
            file_path,
            lyrics_file_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_file_path(
        &mut self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        file_path: &str,
    ) -> Result<(), sqlx::Error> {
        Self::update_file_path(tx, self.uuid, file_path).await?;
        self.file_path = file_path.to_owned();

        Ok(())
    }

    pub async fn uuid_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        lyrics_file_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM lyrics_file
                    WHERE uuid = ?
                )
            "#,
            lyrics_file_uuid,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
