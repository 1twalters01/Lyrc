use sqlx::{Pool, sqlite::Sqlite};
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
        pool: &Pool<Sqlite>,
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
        .fetch_optional(pool)
        .await
    }

    pub async fn select_by_recording_uuid(
        pool: &Pool<Sqlite>,
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
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_source_id(
        pool: &Pool<Sqlite>,
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
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_identifier(
        pool: &Pool<Sqlite>,
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
        .fetch_optional(pool)
        .await
    }

    pub async fn insert(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
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
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &Pool<Sqlite>,
        recording_identifier_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM recording_identifier
                WHERE uuid = ?
            "#,
            recording_identifier_uuid,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uuid_exists(
        pool: &Pool<Sqlite>,
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
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn exists(
        pool: &Pool<Sqlite>,
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
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
