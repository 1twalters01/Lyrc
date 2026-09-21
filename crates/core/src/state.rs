use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};

use mpris::{
    client::MprisClient,
    playback::{PlaybackStatus, PlayerEvent},
    track::Track,
};
use subtitles::{language::Language, subtitles::SubtitleDocument};

use crate::{history::EditHistory, mode::AppMode};

#[derive(Clone)]
pub struct SubtitleDocumentState {
    pub document: SubtitleDocument,
    pub edit_history: EditHistory,
    pub unsaved_changes: bool,
}

impl SubtitleDocumentState {
    pub fn new(document: SubtitleDocument) -> Self {
        Self {
            document,
            edit_history: EditHistory::new(),
            unsaved_changes: false,
        }
    }

    pub fn mark_modified(&mut self) {
        self.unsaved_changes = true;
    }

    pub fn mark_saved(&mut self) {
        self.unsaved_changes = false;
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SubtitleVariant {
    Original,
    Translated(Language),
}

#[derive(Clone)]
pub struct SubtitleDocuments {
    documents: HashMap<SubtitleVariant, SubtitleDocumentState>,
    active_variant: Option<SubtitleVariant>,
}

impl SubtitleDocuments {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            active_variant: None,
        }
    }

    // When config option gets added prioritise those
    pub fn select_default(&mut self) {
        self.active_variant = if self.documents.contains_key(&SubtitleVariant::Original) {
            Some(SubtitleVariant::Original)
        } else {
            None
        };
    }

    pub fn clear(&mut self) {
        self.documents = HashMap::new();
        self.active_variant = None;
    }

    pub fn active(&self) -> Option<&SubtitleDocumentState> {
        match &self.active_variant {
            Some(language) => self.documents.get(&language),
            None => None,
        }
    }

    pub fn active_mut(&mut self) -> Option<&mut SubtitleDocumentState> {
        match &self.active_variant {
            Some(language) => self.documents.get_mut(&language),
            None => None,
        }
    }

    pub fn set_language(&mut self, language: SubtitleVariant) -> bool {
        if self.documents.contains_key(&language) {
            self.active_variant = Some(language);
            true
        } else {
            false
        }
    }

    pub fn insert(
        &mut self,
        subtitle_variant: SubtitleVariant,
        document_state: SubtitleDocumentState,
    ) {
        self.documents.insert(subtitle_variant, document_state);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub track: Option<Track>,
    pub subtitle_documents: SubtitleDocuments,

    pub playback_state: PlaybackStatus,
    pub last_updated: Option<Instant>,

    pub playback_speed: f64,

    /* other app state */
    pub quit: bool,
    pub automatic_scroll_offset: usize,
    pub app_mode: AppMode,
    pub alignment_running: bool,
    pub translation_running: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            track: None,
            subtitle_documents: SubtitleDocuments::new(),
            playback_state: PlaybackStatus::Unknown,
            last_updated: None,
            playback_speed: 1f64,

            quit: false,
            automatic_scroll_offset: 0,
            app_mode: AppMode::Normal,
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
                self.reload_subtitle_documents().await;
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

    pub async fn reload_subtitle_documents(&mut self) {
        self.subtitle_documents.clear();

        let Some(track) = &self.track else { return };

        for (variant, path) in Self::discover_subtitle_files(track) {
            let Ok(document) = SubtitleDocument::from_pathbuf(path) else {
                continue;
            };

            self.subtitle_documents
                .insert(variant, SubtitleDocumentState::new(document));
        }

        self.subtitle_documents.select_default();
    }

    fn discover_subtitle_files(track: &Track) -> Vec<(SubtitleVariant, PathBuf)> {
        let Some(file_path) = &track.file_path else {
            return Vec::new();
        };

        let Some(directory) = file_path.parent() else {
            return Vec::new();
        };

        let Some(track_stem) = file_path.file_stem().and_then(|s| s.to_str()) else {
            return Vec::new();
        };

        let Ok(files) = std::fs::read_dir(directory) else {
            return Vec::new();
        };

        let mut candidates: HashMap<SubtitleVariant, (SubtitleExtension, PathBuf)> = HashMap::new();

        for entry in files.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some((variant, extension)) = classify_subtitle_file(&path, track_stem) else {
                continue;
            };

            match candidates.get(&variant) {
                Some((current_extension, _))
                    if current_extension.priority() >= extension.priority() =>
                {
                    continue;
                }
                _ => {}
            }

            candidates.insert(variant, (extension, path));
        }

        candidates
            .into_iter()
            .map(|(variant, (_, path))| (variant, path))
            .collect()
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

pub enum SubtitleExtension {
    ELRC,
    LRC,
    TXT,
}

impl SubtitleExtension {
    fn priority(&self) -> u8 {
        match self {
            Self::ELRC => 3,
            Self::LRC => 2,
            Self::TXT => 1,
        }
    }
}

fn classify_subtitle_file(
    path: &Path,
    track_stem: &str,
) -> Option<(SubtitleVariant, SubtitleExtension)> {
    let name = path.file_name()?.to_str()?;
    let suffix = name.strip_prefix(track_stem)?.strip_prefix(".")?;

    let parts: Vec<_> = suffix.split(".").collect();
    match parts.as_slice() {
        ["elrc"] => Some((SubtitleVariant::Original, SubtitleExtension::ELRC)),
        ["lrc"] => Some((SubtitleVariant::Original, SubtitleExtension::LRC)),
        ["txt"] => Some((SubtitleVariant::Original, SubtitleExtension::TXT)),
        [language, "lrc"] => Some((
            SubtitleVariant::Translated(Language::from_str(language).ok()?),
            SubtitleExtension::LRC,
        )),
        [language, "txt"] => Some((
            SubtitleVariant::Translated(Language::from_str(language).ok()?),
            SubtitleExtension::TXT,
        )),
        _ => None,
    }
}
