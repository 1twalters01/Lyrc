use chrono::Duration;
use mpris::playback::{PlaybackCommand, PlaybackStatus};
use subtitles::subtitles::{SubtitleCues, SyncLevel};

use crate::{app::App, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn get_current_position(&self) -> Option<Duration> {
        self.mpris_client.get_current_position().await.ok()
    }

    pub async fn get_playback_status(&self) -> PlaybackStatus {
        self.mpris_client
            .get_playback_status()
            .await
            .unwrap_or(PlaybackStatus::Unknown)
    }

    pub async fn toggle_play_pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.mpris_client.execute(PlaybackCommand::Toggle).await?;
        Ok(())
    }

    pub async fn seek_by_duration(
        &mut self,
        duration: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mpris_client
            .execute(PlaybackCommand::Seek(duration))
            .await?;
        Ok(())
    }

    // Use a better error type
    pub async fn seek_to_selected_line(
        &mut self,
        cue_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.state.track.is_none() {
            return Err(String::from("No track was found").into());
        }

        if let Some(ref document) = self.state.subtitle_document {
            if document.sync_level() == SyncLevel::None {
                return Ok(());
            }

            let duration = match &document.cues {
                SubtitleCues::Word(cues) => cues[cue_index].start,
                SubtitleCues::Cue(cues) => cues[cue_index].start,
                SubtitleCues::Line(_) => return Err(String::from("No subtitle times").into()),
                SubtitleCues::None => return Err(String::from("No subtitle cues").into()),
            };
            self.mpris_client
                .execute(PlaybackCommand::SetPosition(duration))
                .await?;
        } else {
            return Err(String::from("No subtitle_document was found").into());
        }

        Ok(())
    }
}
