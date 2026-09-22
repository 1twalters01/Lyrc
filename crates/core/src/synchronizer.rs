use chrono::Duration;
use subtitles::subtitles::{SubtitleCues, SubtitleDocument};
use synchronizer::{
    strategies::{
        cues::{CueIndex, CueSyncEvent, CueSynchronizer},
        words::{WordIndex, WordSyncEvent, WordSynchronizer},
    },
    traits::{CueIndexed, Synchronizer},
};

use crate::state::SubtitleDocumentState;

#[derive(Clone, Debug)]
pub enum SyncEvent {
    Word(WordSyncEvent),
    Cue(CueSyncEvent),
}

#[derive(Clone, Debug)]
pub enum ActiveIndex {
    Cue(CueIndex),
    Word(WordIndex),
}

impl ActiveIndex {
    pub fn cue_index(&self) -> CueIndex {
        match self {
            Self::Word(w) => w.cue_index(),
            Self::Cue(c) => c.cue_index(),
        }
    }
}

impl Default for ActiveIndex {
    fn default() -> Self {
        Self::Cue(CueIndex::default())
    }
}

#[derive(Debug)]
pub enum SynchronizerMode {
    Cue,
    Word,
    None,
}

pub struct AppSynchronizer {
    pub cue_synchronizer: CueSynchronizer,
    pub word_synchronizer: WordSynchronizer,
    pub mode: SynchronizerMode,
}

impl AppSynchronizer {
    pub fn new() -> Self {
        let cue_synchronizer = CueSynchronizer::new();
        let word_synchronizer = WordSynchronizer::new();

        Self {
            cue_synchronizer,
            word_synchronizer,
            mode: SynchronizerMode::Cue,
        }
    }

    pub fn get_active_indices(&self) -> Vec<ActiveIndex> {
        match self.mode {
            SynchronizerMode::Word => self
                .word_synchronizer
                .get_active_indices()
                .iter()
                .map(|index| ActiveIndex::Word(*index))
                .collect(),
            SynchronizerMode::Cue => self
                .cue_synchronizer
                .get_active_indices()
                .iter()
                .map(|index| ActiveIndex::Cue(*index))
                .collect(),
            SynchronizerMode::None => Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        subtitle_document_state: &Option<&SubtitleDocumentState>,
        position: &Option<Duration>,
    ) -> Option<SyncEvent> {
        let subtitle_document: &Option<SubtitleDocument> =
            &subtitle_document_state.map(|state| state.document.clone());

        if let Some(document) = subtitle_document {
            match document.cues {
                SubtitleCues::Word(_) => self.mode = SynchronizerMode::Word,
                SubtitleCues::Cue(_) => self.mode = SynchronizerMode::Cue,
                SubtitleCues::Line(_) => self.mode = SynchronizerMode::Cue,
                SubtitleCues::None => self.mode = SynchronizerMode::Cue,
            }
        }
        let event = match self.mode {
            SynchronizerMode::Word => self
                .word_synchronizer
                .update(subtitle_document, position)
                .map(|e| SyncEvent::Word(e)),
            SynchronizerMode::Cue => self
                .cue_synchronizer
                .update(subtitle_document, position)
                .map(|e| SyncEvent::Cue(e)),
            SynchronizerMode::None => None,
        };
        return event;
    }
}
