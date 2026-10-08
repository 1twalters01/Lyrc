use sqlx::{Pool, sqlite::Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct LyricsVariantRow {
    pub id: i64,
    pub translation_id: i64,
    pub language_id: i64,
}

impl LyricsVariantRow {
    pub async fn select_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id,
                    translation_id,
                    language_id
                FROM lyrics_variant
                WHERE id = ?
            "#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn select(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        translation_id: i64,
        language_id: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id,
                    translation_id,
                    language_id
                FROM lyrics_variant
                WHERE translation_id = ?
                  AND language_id = ?
            "#,
            translation_id,
            language_id,
        )
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn select_by_translation_id(
        pool: &Pool<Sqlite>,
        translation_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id,
                    translation_id,
                    language_id
                FROM lyrics_variant
                WHERE translation_id = ?
            "#,
            translation_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn select_by_language_id(
        pool: &Pool<Sqlite>,
        language_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id,
                    translation_id,
                    language_id
                FROM lyrics_variant
                WHERE language_id = ?
            "#,
            language_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn insert_force(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO lyrics_variant (
                    id,
                    translation_id,
                    language_id
                )
                VALUES (?, ?, ?)
            "#,
            self.id,
            self.translation_id,
            self.language_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn insert(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query!(
            r#"
                INSERT INTO lyrics_variant (
                    translation_id,
                    language_id
                )
                VALUES (?, ?)
            "#,
            self.translation_id,
            self.language_id,
        )
        .execute(&mut **tx)
        .await?;

        Ok(res.last_insert_rowid())
    }

    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM lyrics_variant
                WHERE id = ?
            "#,
            id,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn id_exists(
        pool: &Pool<Sqlite>,
        lyrics_variant_id: i32,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM lyrics_variant
                    WHERE id = ?
                )
            "#,
            lyrics_variant_id,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }

    pub async fn exists(
        pool: &Pool<Sqlite>,
        translation_id: i64,
        language_id: i64,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM lyrics_variant
                    WHERE translation_id = ?
                      AND language_id = ?
                )
            "#,
            translation_id,
            language_id,
        )
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
    }
}
