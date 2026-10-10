use uuid::Uuid;

#[derive(Debug)]
pub struct AudioFileRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub file_path: String,
}

impl AudioFileRow {
    pub async fn select_by_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uuid: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    file_path
                FROM audio_file
                WHERE uuid = ?
            "#,
            uuid,
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
                    file_path
                FROM audio_file
                WHERE recording_uuid = ?
            "#,
            recording_uuid,
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
                    file_path
                FROM audio_file
                WHERE file_path = ?
            "#,
            file_path,
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
                INSERT INTO audio_file (
                    uuid,
                    recording_uuid,
                    file_path
                )
                VALUES (?, ?, ?)
            "#,
            self.uuid,
            self.recording_uuid,
            self.file_path,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn update_file_path(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uuid: Uuid,
        file_path: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE audio_file
            SET file_path = ?
            WHERE uuid = ?
        "#,
            file_path,
            uuid,
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

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM audio_file
                WHERE uuid = ?
            "#,
            uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        audio_file_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM audio_file
                    WHERE uuid = ?
                )
            "#,
            audio_file_uuid,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_uuid: Uuid,
        file_path: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM audio_file
                    WHERE recording_uuid = ?
                      AND file_path = ?
                )
            "#,
            recording_uuid,
            file_path,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
