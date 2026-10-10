use std::{path::PathBuf, str::FromStr};

use database::models::{
    audio_file::AudioFileRow, lookup, lyrics_file::LyricsFileRow, lyrics_variant::LyricsVariantRow,
};
use subtitles::subtitles::SubtitleDocument;

use crate::{
    app::App,
    renderer::Renderer,
    state::{SubtitleDocumentState, SubtitleVariant},
};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn update_track(&mut self) {
        self.state
            .update_track(self.mpris_client.get_current_track().await.ok())
            .await;
    }

    pub async fn update_subtitle_document(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.reload_subtitle_documents().await;

        let mut tx = self.database_service.get_pool().begin().await?;
        let track = match self.state.track {
            Some(ref track) => track,
            None => return Err(String::from("No track found").into()),
            // None => return Ok(()),
        };
        let audio_file_path = match track.file_path {
            Some(ref file_path) => file_path,
            None => return Ok(()),
        };

        // Get audio file from db using audio file path
        let audio_file_row = match AudioFileRow::select_by_file_path(
            &mut tx,
            &audio_file_path.to_string_lossy().to_string(),
        )
        .await?
        {
            Some(audio_file_row) => audio_file_row,
            None => return Ok(()),
        };
        // println!("\naudio_file_row: {:#?}", audio_file_row);

        let lyrics_file_rows =
            LyricsFileRow::select_by_recording_uuid(&mut tx, audio_file_row.recording_uuid).await?;
        // println!("\nlyrics_file_row: {:#?}", lyrics_file_rows);

        if lyrics_file_rows.is_empty() {
            return Ok(());
        };

        for lyrics_file_row in lyrics_file_rows {
            let subtitle_document =
                SubtitleDocument::from_pathbuf(PathBuf::from_str(&lyrics_file_row.file_path)?)?;
            let subtitle_document_state = SubtitleDocumentState::new(subtitle_document);
            let subtitle_variant_row =
                match LyricsVariantRow::select_by_id(&mut tx, lyrics_file_row.variant_id).await? {
                    Some(row) => row,
                    None => return Ok(()),
                };
            // println!("\n\nsubtitle_variant_row: {:#?}",
            // subtitle_variant_row);

            let language = match lookup::get_language_by_id(
                &mut tx,
                subtitle_variant_row.language_id,
            )
            .await?
            {
                Some(language) => language,
                None => return Ok(()),
            };
            // println!("language: {}", language);
            let subtitle_variant =
                match lookup::get_translation_by_id(&mut tx, subtitle_variant_row.translation_id)
                    .await?
                {
                    Some(translation) => match translation.as_str() {
                        "original" => SubtitleVariant::Original,
                        "translated" => SubtitleVariant::Translated(language),
                        _ => return Ok(()),
                    },
                    None => return Ok(()),
                };
            println!("subtitle_variant: {:?}", subtitle_variant);
            self.state
                .subtitle_documents
                .insert_forced(subtitle_variant, subtitle_document_state);
            // println!("active_variant: {:?}",
            // self.state.subtitle_documents.active_variant);
        }
        // If not possible then:
        self.state.subtitle_documents.select_default();
        Ok(())
    }
}
