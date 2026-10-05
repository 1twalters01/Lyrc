use std::{cmp::min, collections::BTreeSet, fmt::Display};

use subtitles::subtitles::SubtitleCues;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectCursor {
    pub cue_index: usize,
    pub active_word_index: Option<usize>,
    pub target_word_index: Option<usize>,
}

impl SelectCursor {
    pub fn new(cue_index: usize, use_words: bool) -> Self {
        let (active_word_index, target_word_index) = if use_words {
            (Some(0), Some(0))
        } else {
            (None, None)
        };

        Self {
            cue_index,
            active_word_index,
            target_word_index,
        }
    }

    pub fn move_left(&mut self) {
        self.active_word_index = self.active_word_index.map(|idx| idx.saturating_sub(1));
        self.target_word_index = self.active_word_index;
    }

    pub fn move_right(&mut self, word_count: Option<usize>) {
        self.active_word_index = min(
            self.active_word_index.map(|idx| idx + 1),
            word_count.map(|c| c.saturating_sub(1)),
        );
        self.target_word_index = self.active_word_index;
    }

    pub fn end(&mut self, word_count: Option<usize>) {
        self.active_word_index = word_count.map(|c| c.saturating_sub(1));
    }

    pub fn home(&mut self) {
        self.active_word_index = match self.active_word_index {
            Some(_) => Some(0),
            None => None,
        };
    }

    pub fn move_up(&mut self, new_word_count: Option<usize>) {
        self.cue_index = self.cue_index.saturating_sub(1);
        self.active_word_index = min(
            self.target_word_index,
            new_word_count.map(|idx| idx.saturating_sub(1)),
        );
    }

    pub fn move_down(&mut self, new_word_count: Option<usize>, line_count: usize) {
        self.cue_index = min(
            self.cue_index.saturating_add(1),
            line_count.saturating_sub(1),
        );
        self.active_word_index = min(
            self.target_word_index,
            new_word_count.map(|idx| idx.saturating_sub(1)),
        );
    }
}

#[derive(Clone, PartialEq)]
pub struct EditCue {
    pub index: usize,
    pub original_content: SubtitleCues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditCursor {
    pub cue_index: usize,
    pub active_column: usize,
    pub target_column: usize,
}

impl EditCursor {
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

    pub fn end(&mut self, line_length: usize) {
        self.active_column = line_length;
    }

    pub fn home(&mut self) {
        self.active_column = 0;
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
        selected_cues: Vec<usize>,
        cursor: SelectCursor,
    },
    Edit {
        selected_cues: Vec<EditCue>,
        cursor: EditCursor,
    },
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Select {
                selected_cues: _,
                cursor:
                    SelectCursor {
                        cue_index,
                        active_word_index,
                        target_word_index: _,
                    },
            } => write!(
                f,
                "select cue: {:?}, active_column: {:?}",
                cue_index, active_word_index
            ),
            Self::Edit {
                selected_cues: _,
                cursor:
                    EditCursor {
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
