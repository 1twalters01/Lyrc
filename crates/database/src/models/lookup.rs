use std::str::FromStr;

use subtitles::language::Language;

pub async fn get_id_by_lyrics_format(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    format: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
            SELECT id
            FROM lyrics_format
            WHERE format = ?
        "#,
        format,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_id_by_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
            SELECT id
            FROM source
            WHERE source = ?
        "#,
        source,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_id_by_translation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    name: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
            SELECT id
            FROM translation
            WHERE name = ?
        "#,
        name,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_translation_by_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    translation_id: i64,
) -> Result<Option<String>, sqlx::Error> { 
    sqlx::query_scalar!(
        r#"
            SELECT name
            FROM translation
            WHERE id = ?
        "#,
        translation_id,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_id_by_language_code(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    code_3: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
            SELECT id
            FROM language
            WHERE code_3 = ?
        "#,
        code_3,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_id_by_language(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    language: &Language,
) -> Result<Option<i64>, sqlx::Error> {
    let code_3 = language.as_code_3();
    sqlx::query_scalar!(
        r#"
            SELECT id
            FROM language
            WHERE code_3 = ?
        "#,
        code_3,
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_language_by_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    language_id: i64,
) -> Result<Option<Language>, Box<dyn std::error::Error>> {
    let code_3 = sqlx::query_scalar!(
        r#"
                SELECT code_3
                FROM language
            WHERE id = ?
        "#,
        language_id
    )
        .fetch_optional(&mut **tx)
    .await?;

    match code_3 {
        Some(code_3) => Ok(Some(Language::from_str(&code_3)?)),
        None => Ok(None),
    }
}
