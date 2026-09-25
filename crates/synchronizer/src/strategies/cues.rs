use chrono::Duration;
use subtitles::subtitles::{SubtitleCues, SubtitleDocument};

use crate::traits::{ActiveIndexed, CueIndexed, Synchronizer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CueIndex {
    pub cue: usize,
}

impl Default for CueIndex {
    fn default() -> Self {
        Self { cue: 0 }
    }
}

impl CueIndexed for CueIndex {
    fn cue_index(&self) -> Self {
        *self
    }
}

impl ActiveIndexed for CueIndex {}

#[derive(Clone, Debug)]
pub enum CueSyncEvent {
    Changed {
        old_cues: Vec<CueIndex>,
        new_cues: Vec<CueIndex>,
    },
}

pub struct CueSynchronizer {
    active_cues: Vec<CueIndex>,
    last_position: Option<Duration>,
}

impl Synchronizer for CueSynchronizer {
    type Event = CueSyncEvent;
    type Active = CueIndex;

    fn update(
        &mut self,
        subtitle_document: &Option<SubtitleDocument>,
        position: &Option<Duration>,
    ) -> Option<Self::Event> {
        let (subtitle_document, position) = match (subtitle_document, position) {
            (Some(subtitle_document), Some(position)) => (subtitle_document, position),
            (_, _) => return None,
        };

        let is_seeking_backwards = self.last_position.is_some_and(|last| position <= &last);

        let mut new_cues = Self::get_cues_at(&subtitle_document, Some(position));
        new_cues = if new_cues.is_empty() && !is_seeking_backwards {
            self.active_cues.clone()
        } else {
            new_cues
        };

        self.last_position = Some(position.clone());

        if new_cues != self.active_cues {
            let old_cues = std::mem::replace(&mut self.active_cues, new_cues);

            let event = CueSyncEvent::Changed {
                old_cues,
                new_cues: self.active_cues.clone(),
            };

            return Some(event);
        }

        None
    }

    fn get_active_indices(&self) -> &[CueIndex] {
        &self.active_cues
    }
}

impl CueSynchronizer {
    pub fn new() -> Self {
        Self {
            active_cues: Vec::new(),
            last_position: None,
        }
    }

    pub fn get_cues_at(
        subtitle_document: &SubtitleDocument,
        position: Option<&Duration>,
    ) -> Vec<CueIndex> {
        let position = match position {
            Some(position) => position,
            None => return Vec::new(),
        };

        match &subtitle_document.cues {
            SubtitleCues::Word(cues) => {
                let start = cues.partition_point(|cue| &cue.start <= position);
                cues[..start]
                    .iter()
                    .enumerate()
                    .filter_map(|(index, cue)| {
                        if position < &cue.end {
                            Some(CueIndex { cue: index })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<CueIndex>>()
            }
            SubtitleCues::Cue(cues) => {
                let start = cues.partition_point(|cue| &cue.start <= position);
                cues[..start]
                    .iter()
                    .enumerate()
                    .filter_map(|(index, cue)| {
                        if position < &cue.end {
                            Some(CueIndex { cue: index })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<CueIndex>>()
            }
            SubtitleCues::Line(_) => Vec::new(),
            SubtitleCues::None => Vec::new(),
        }
    }
}
