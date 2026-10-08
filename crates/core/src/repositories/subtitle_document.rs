use std::path::PathBuf;

use chrono::Duration;
use sqlx::{Pool, Sqlite};
use subtitles::subtitles::SubtitleDocument;

use database::models::{
    artist::ArtistRow, audio_file::AudioFileRow, lookup, lyrics_file::LyricsFileRow, lyrics_variant::LyricsVariantRow, recording::RecordingRow, recording_artist::RecordingArtistRow, song::SongRow, song_artist::SongArtistRow,
};

use crate::state::SubtitleVariant;

pub struct SubtitleDocumentRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> SubtitleDocumentRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        document: &SubtitleDocument,
        track_duration: Duration,
        variant: SubtitleVariant,
        audio_file_path: &PathBuf,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let artist_names = &document.metadata.artists;
        let song_title = match &document.metadata.title {
            Some(song_title) => song_title,
            None => return Err(sqlx::Error::InvalidArgument(String::from("No song title"))),
        };

        let artist_rows = ArtistRow::get_or_create_by_names(&mut tx, artist_names).await?;
        let artist_row = &artist_rows[0];

        let song_row = if SongArtistRow::exists_by_name(&mut tx, song_title, &artist_names[0])
            .await?
        {
            SongRow::get_or_create_by_titles(&mut tx, &[song_title.to_string()]).await?[0].clone()
        } else {
            let song_row = SongRow::from_title(song_title);
            song_row.insert(&mut tx).await?;
            let song_artist_row = SongArtistRow {
                song_uuid: song_row.uuid,
                artist_uuid: artist_row.uuid,
            };
            song_artist_row.insert(&mut tx).await?;
            song_row
        };

        let recording_rows = RecordingRow::select_by_song_uuid(&mut tx, song_row.uuid).await?;
        let recording_row = match recording_rows.first() {
            Some(row) => row.clone(),
            None => {
                let recording_row = RecordingRow {
                    uuid: uuid::Uuid::new_v4(),
                    song_uuid: song_row.uuid,
                    duration_ms: track_duration.num_milliseconds(),
                };

                recording_row.insert(&mut tx).await?;

                recording_row
            }
        };

        let recording_artist_row = match RecordingArtistRow::select_by_uuids(&mut tx, recording_row.uuid, artist_row.uuid).await? {
            Some(row) => row,
            None => {
                let row = RecordingArtistRow {
                    recording_uuid: recording_row.uuid,
                    artist_uuid: artist_row.uuid,
                };

                row.insert(&mut tx).await?;

                row
            },
        };

        // Get relevant Source IDs
        
        // Try to insert to Recording Identifier

        // Get lyrics Variant
        let translation = match variant {
            SubtitleVariant::Original => "original",
            SubtitleVariant::Translated(_) => "translated",
        };
        let translation_id = lookup::get_id_by_translation(&mut tx, translation)
            .await?
            .unwrap();

        let language = document.metadata.languages[0];
        let language_id = lookup::get_id_by_language(&mut tx, &language)
            .await?
            .unwrap();

        let lyrics_variant =
            match LyricsVariantRow::select(&mut tx, translation_id, language_id).await? {
                Some(variant) => variant,
                None => {
                    let mut lyrics_variant = LyricsVariantRow {
                        id: 0,
                        translation_id,
                        language_id,
                    };
                    let id = lyrics_variant.insert(&mut tx).await?;
                    lyrics_variant.id = id;
                    lyrics_variant
                }
            };

        // Get format format id
        let document_file_path = document.metadata.file_path.clone().unwrap();
        let format = document.sync_level().as_str();
        let format_id = lookup::get_id_by_lyrics_format(&mut tx, format)
            .await?
            .unwrap();

        // Lyrics File
        let lyrics_file_row =
            match LyricsFileRow::select(&mut tx, recording_row.uuid, lyrics_variant.id, format_id)
                .await?
            {
                Some(lyrics_file_row) => lyrics_file_row,
                None => {
                    let lyrics_file_row = LyricsFileRow {
                        uuid: uuid::Uuid::new_v4(),
                        recording_uuid: recording_row.uuid,
                        variant_id: lyrics_variant.id,
                        format_id: format_id,
                        file_path: document_file_path.to_string_lossy().to_string(),
                    };

                    lyrics_file_row.insert(&mut tx).await?;

                    lyrics_file_row
                }
            };
        println!("lyrics_file_row: {:?}", lyrics_file_row);

        // Need to save audio file too so that loading from a given audiofile works
        let audio_file_path_string = audio_file_path.to_string_lossy().to_string();
        let audio_file_row = match AudioFileRow::select_by_file_path(&mut tx, &audio_file_path_string).await? {
            Some(audio_file_row) => audio_file_row,
            None => {
                let audio_file_row = AudioFileRow {
                    uuid: uuid::Uuid::new_v4(),
                    recording_uuid: recording_row.uuid,
                    file_path: audio_file_path_string,
                };
                audio_file_row.insert(&mut tx).await?;
                audio_file_row
            }
        };

        tx.commit().await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<SubtitleDocument, sqlx::Error> {
        todo!()
    }

    pub async fn delete(&self, document: &SubtitleDocument) -> Result<bool, sqlx::Error> {
        todo!()
    }
}
