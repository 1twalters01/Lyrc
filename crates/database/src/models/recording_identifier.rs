use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct RecordingIdentifierRow {
    pub uuid: Uuid,
    pub recording_uuid: Uuid,
    pub source_id: i64,
    pub identifier: String,
}

impl RecordingIdentifierRow {
    pub async fn select_by_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_identifier_uuid: Uuid,
    ) -> Result<Option<RecordingIdentifierRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingIdentifierRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    source_id,
                    identifier
                FROM recording_identifier
                WHERE uuid = ?
            "#,
            recording_identifier_uuid,
        )
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_recording_uuid(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_uuid: Uuid,
    ) -> Result<Vec<RecordingIdentifierRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingIdentifierRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    source_id,
                    identifier
                FROM recording_identifier
                WHERE recording_uuid = ?
            "#,
            recording_uuid,
        )
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_source_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        source_id: i64,
    ) -> Result<Vec<RecordingIdentifierRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingIdentifierRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    source_id,
                    identifier
                FROM recording_identifier
                WHERE source_id = ?
            "#,
            source_id,
        )
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn select_by_identifier(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        source_id: i64,
        identifier: &str,
    ) -> Result<Option<RecordingIdentifierRow>, sqlx::Error> {
        sqlx::query_as!(
            RecordingIdentifierRow,
            r#"
                SELECT
                    uuid AS "uuid: uuid::Uuid",
                    recording_uuid AS "recording_uuid: uuid::Uuid",
                    source_id,
                    identifier
                FROM recording_identifier
                WHERE source_id = ?
                  AND identifier = ?
            "#,
            source_id,
            identifier,
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
                INSERT INTO recording_identifier (
                    uuid,
                    recording_uuid,
                    source_id,
                    identifier
                )
                VALUES (?, ?, ?, ?)
            "#,
            self.uuid,
            self.recording_uuid,
            self.source_id,
            self.identifier,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_identifier_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM recording_identifier
                WHERE uuid = ?
            "#,
            recording_identifier_uuid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recording_identifier_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM recording_identifier
                    WHERE uuid = ?
                )
            "#,
            recording_identifier_uuid,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        source_id: i64,
        identifier: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM recording_identifier
                    WHERE source_id = ?
                      AND identifier = ?
                )
            "#,
            source_id,
            identifier,
        )
        .fetch_one(&mut **tx)
        .await
        .map(|exists| exists != 0)
    }
}
