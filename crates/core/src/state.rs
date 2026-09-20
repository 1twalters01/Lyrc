use std::time::Instant;

use mpris::{
    client::MprisClient,
    playback::{PlaybackStatus, PlayerEvent},
    track::Track,
};
use subtitles::subtitles::SubtitleDocument;

use crate::{history::EditHistory, mode::AppMode};

#[derive(Clone)]
pub struct AppState {
    pub track: Option<Track>,
    pub subtitle_document: Option<SubtitleDocument>,
    pub edit_history: EditHistory,
    pub playback_state: PlaybackStatus,
    pub last_updated: Option<Instant>,

    /* Add event for change in playback speed */
    pub playback_speed: f64,

    /* other app state */
    pub quit: bool,
    pub automatic_scroll_offset: usize,
    pub app_mode: AppMode,
    pub unsaved_changes: bool,
    pub alignment_running: bool,
    pub translation_running: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            track: None,
            subtitle_document: None,
            edit_history: EditHistory::new(),
            playback_state: PlaybackStatus::Unknown,
            last_updated: None,
            playback_speed: 1f64,

            quit: false,
            automatic_scroll_offset: 0,
            app_mode: AppMode::Normal,
            unsaved_changes: false,
            alignment_running: false,
            translation_running: false,
        }
    }

    pub async fn update(
        &mut self,
        mpris: &mut MprisClient,
        event: &PlayerEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            PlayerEvent::TrackChanged(track) => {
                if self.track.as_ref() != Some(track) {
                    self.app_mode = AppMode::Normal
                }

                self.update_track(Some(track.to_owned())).await;
                self.update_subtitle_document().await;
            }
            PlayerEvent::PlaybackChanged(playback) => {
                self.playback_state = playback.clone();
                if playback == &PlaybackStatus::Stopped {
                    let targets = Vec::from([String::from("mpv"), String::from("cmus")]);
                    let player = MprisClient::choose_player(&targets).await?;
                    *mpris = MprisClient::connect(&player).await?;
                    self.playback_state = mpris.get_playback_status().await?;
                }
            }
            PlayerEvent::Seeked(_duration) => {}
        }

        Ok(())
    }

    pub async fn update_track(&mut self, current_track: Option<Track>) {
        self.track = current_track;
    }

    pub async fn update_subtitle_document(&mut self) {
        self.subtitle_document = match self.track {
            Some(ref track) => {
                let mut document = None;

                for path in [
                    track.get_elrc_file_path(),
                    track.get_lrc_file_path(),
                    track.get_txt_file_path(),
                ]
                .into_iter()
                .flatten()
                {
                    if path.exists() {
                        if let Ok(value) = SubtitleDocument::from_pathbuf(path) {
                            document = Some(value);
                            break;
                        }
                    }
                }

                document
            }
            None => None,
        };
    }

    pub fn is_normal_mode(&self) -> bool {
        match self.app_mode {
            AppMode::Normal => true,
            _ => false,
        }
    }
    pub fn is_select_mode(&self) -> bool {
        match self.app_mode {
            AppMode::Select {
                cue_index: _,
                selected_cues: _,
            } => true,
            _ => false,
        }
    }
    pub fn is_edit_mode(&self) -> bool {
        match self.app_mode {
            AppMode::Edit {
                cursor: _,
                selected_cues: _,
            } => true,
            _ => false,
        }
    }
}
