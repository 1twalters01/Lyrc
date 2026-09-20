use std::{cmp::min, collections::BTreeSet, fmt::Display};

use subtitles::subtitles::SubtitleCues;

#[derive(Clone, PartialEq)]
pub struct EditCue {
    pub index: usize,
    pub original_content: SubtitleCues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub cue_index: usize,
    pub active_column: usize,
    pub target_column: usize,
}

impl Cursor {
    pub fn new(cue_index: usize, line_length: usize) -> Self {
        Self {
            cue_index,
            active_column: line_length,
            target_column: line_length,
        }
    }

    pub fn move_left(&mut self) {
        self.active_column = self.active_column.saturating_sub(1);
        self.target_column = self.active_column;
    }

    pub fn move_right(&mut self, line_length: usize) {
        self.active_column = min(self.active_column + 1, line_length);
        self.target_column = self.active_column;
    }

    pub fn move_up(&mut self, new_line_length: usize) {
        self.cue_index = self.cue_index.saturating_sub(1);
        self.active_column = min(self.target_column, new_line_length);
    }

    pub fn move_down(&mut self, new_line_length: usize, line_count: usize) {
        self.cue_index = min(
            self.cue_index.saturating_add(1),
            line_count.saturating_sub(1),
        );
        self.active_column = min(self.target_column, new_line_length);
    }

    pub fn jump_up(&mut self, new_line_length: usize, selected_cues: &BTreeSet<usize>) {
        self.cue_index = selected_cues
            .iter()
            .rev()
            .find(|&&c| c < self.cue_index)
            .copied()
            .unwrap_or(self.cue_index);
        self.active_column = min(self.target_column, new_line_length);
    }

    pub fn jump_down(&mut self, new_line_length: usize, selected_cues: &BTreeSet<usize>) {
        self.cue_index = selected_cues
            .iter()
            .find(|&&c| c > self.cue_index)
            .copied()
            .unwrap_or(self.cue_index);
        self.active_column = min(self.target_column, new_line_length);
    }
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Select {
        cue_index: usize,
        selected_cues: Vec<usize>,
        // selected_cues: BTreeSet<usize>,
    },
    Edit {
        cursor: Cursor,
        selected_cues: Vec<EditCue>,
    },
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Select {
                cue_index,
                selected_cues: _,
            } => write!(f, "select cue: {:?}", cue_index),
            Self::Edit {
                selected_cues: _,
                cursor:
                    Cursor {
                        cue_index,
                        active_column,
                        target_column: _,
                    },
            } => write!(
                f,
                "cursor index: {:?}, active_column: {:?}",
                cue_index, active_column
            ),
        }
    }
}
