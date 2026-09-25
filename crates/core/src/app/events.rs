use alignment::messages::AlignmentResult;
use chrono::Duration;
use mpris::playback::{PlaybackStatus, PlayerEvent};
use subtitles::subtitles::SyncLevel;
use translation::messages::TranslationResult;

use crate::{
    app::App,
    renderer::Renderer,
    state::{SubtitleDocumentState, SubtitleVariant},
    synchronizer::SynchronizerMode,
};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn handle_player_event(
        &mut self,
        event: PlayerEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.state.update(&mut self.mpris_client, &event).await?;
        self.clock.update(event);
        self.synchronizer.update(
            &self.state.subtitle_documents.active(),
            &self.clock.get_position(),
        );

        self.render()?;
        Ok(())
    }

    pub fn handle_alignment_event(
        &mut self,
        event: AlignmentResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            AlignmentResult::Complete(subtitle_document) => {
                self.state.alignment_running = false;
                if let Some(subtitle_document) = subtitle_document {
                    let new_sync_level = subtitle_document.sync_level();
                    let insert_res = self.state.subtitle_documents.insert(
                        SubtitleVariant::Original,
                        SubtitleDocumentState::new(subtitle_document.clone()),
                    );
                    if insert_res {
                        match new_sync_level {
                            SyncLevel::Word => self.synchronizer.mode = SynchronizerMode::Word,
                            SyncLevel::Cue => self.synchronizer.mode = SynchronizerMode::Cue,
                            _ => self.synchronizer.mode = SynchronizerMode::None,
                        }
                    }

                    self.state
                        .subtitle_documents
                        .set_variant(SubtitleVariant::Original);
                }
            }
            AlignmentResult::Cancelled => {
                self.state.alignment_running = false;
            }
            AlignmentResult::Failed(error) => {
                self.state.alignment_running = false;
                return Err(Box::new(error));
            }
        }

        Ok(())
    }

    pub fn handle_translation_event(
        &mut self,
        event: TranslationResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            TranslationResult::Complete(result) => {
                self.state.translation_running = false;
                if let Some((subtitle_document, language)) = result {
                    let new_sync_level = subtitle_document.sync_level();
                    let insert_res = self.state.subtitle_documents.insert(
                        SubtitleVariant::Translated(language),
                        SubtitleDocumentState::new(subtitle_document),
                    );
                    if insert_res {
                        match new_sync_level {
                            SyncLevel::Word => self.synchronizer.mode = SynchronizerMode::Word,
                            SyncLevel::Cue => self.synchronizer.mode = SynchronizerMode::Cue,
                            _ => self.synchronizer.mode = SynchronizerMode::None,
                        }
                    }

                    self.state
                        .subtitle_documents
                        .set_variant(SubtitleVariant::Translated(language));

                    self.state.modal = None;
                }
            }
            TranslationResult::Cancelled => {
                self.state.translation_running = false;
            }
            TranslationResult::Failed(error) => {
                self.state.translation_running = false;
                return Err(Box::new(error));
            }
        }

        Ok(())
    }

    pub async fn handle_tick_event(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_position = self.get_current_position().await;
        let playback_status = self.get_playback_status().await;
        self.process_tick(current_position, playback_status)
    }

    fn process_tick(
        &mut self,
        current_position: Option<Duration>,
        playback_status: PlaybackStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.clock.sync(current_position, playback_status)?;

        let subtitle_document = &self.state.subtitle_documents.active();
        let position = self.clock.get_position();
        self.synchronizer.update(subtitle_document, &position);

        self.render()?;
        Ok(())
    }

    fn render(&mut self) -> Result<(), R::Error> {
        let position = self.clock.get_position();
        let active_cues = self.synchronizer.get_active_indices();
        self.renderer
            .render(&mut self.state, position, &active_cues)
    }
}
