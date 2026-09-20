use subtitles::subtitles::SubtitleCues;

use crate::{
    app::App,
    mode::{AppMode, Cursor},
    renderer::Renderer,
    synchronizer::ActiveIndex,
};

impl<R> App<R>
where
    R: Renderer,
{
    pub fn get_first_active_index(&self) -> Option<ActiveIndex> {
        match &self.state.subtitle_document {
            Some(_document) => Some(
                self.synchronizer
                    .get_active_indices()
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ),
            None => None,
        }
    }

    pub fn get_first_active_cue_index(&self) -> Option<usize> {
        match &self.state.subtitle_document {
            Some(_document) => match self
                .synchronizer
                .get_active_indices()
                .iter()
                .map(|i| i.cue_index().cue)
                .collect::<Vec<_>>()
                .first()
            {
                Some(line_idx) => Some(line_idx.clone()),
                None => Some(0),
            },
            None => None,
        }
    }

    // Edit mode should go to previous edit line rather than previous line
    pub fn go_to_previous_line(&mut self) {
        if self.state.track.is_none() || self.state.subtitle_document.is_none() {
            self.switch_to_normal_mode();
        }

        let subtitle_document = match &self.state.subtitle_document {
            Some(document) => document,
            None => return,
        };

        let app_mode = match &self.state.app_mode {
            AppMode::Normal => match self.get_first_active_cue_index() {
                Some(index) => AppMode::Select {
                    cue_index: index,
                    selected_cues: Vec::new(),
                },
                None => AppMode::Normal,
            },
            AppMode::Select {
                cue_index,
                selected_cues,
            } => AppMode::Select {
                cue_index: cue_index.saturating_sub(1),
                selected_cues: selected_cues.clone(),
            },
            AppMode::Edit {
                cursor,
                selected_cues,
            } => {
                let mut new_cursor = cursor.clone();
                let new_line_length = subtitle_document
                    .cues
                    .cue_len(new_cursor.cue_index.saturating_sub(1));
                new_cursor.move_up(new_line_length);
                AppMode::Edit {
                    cursor: new_cursor,
                    selected_cues: selected_cues.clone(),
                }
            }
        };

        self.state.app_mode = app_mode;
    }

    // Edit mode should go to next edit line rather than next line
    pub fn go_to_next_line(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
            return;
        }

        let app_mode = match &self.state.app_mode {
            AppMode::Normal => match self.get_first_active_cue_index() {
                Some(index) => AppMode::Select {
                    cue_index: index,
                    selected_cues: Vec::new(),
                },
                None => AppMode::Normal,
            },
            AppMode::Select {
                cue_index,
                selected_cues,
            } => match &self.state.subtitle_document {
                Some(subtitle_document) => match &subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - 1 {
                            index += 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - 1 {
                            index += 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Line(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - 1 {
                            index += 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::None => return,
                },
                None => AppMode::Normal,
            },
            AppMode::Edit {
                cursor,
                selected_cues,
            } => match &self.state.subtitle_document {
                Some(subtitle_document) => match &subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - 1 {
                            new_index += 1;
                        }

                        let new_line_length = subtitle_document.cues.cue_len(new_index);
                        let line_count = cues.len();

                        let mut new_cursor = cursor.clone();
                        new_cursor.move_down(new_line_length, line_count);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - 1 {
                            new_index += 1;
                        }

                        let new_line_length = subtitle_document.cues.cue_len(new_index);
                        let line_count = cues.len();

                        let mut new_cursor = cursor.clone();
                        new_cursor.move_down(new_line_length, line_count);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Line(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - 1 {
                            new_index += 1;
                        }

                        let new_line_length = subtitle_document.cues.cue_len(new_index);
                        let line_count = cues.len();

                        let mut new_cursor = cursor.clone();
                        new_cursor.move_down(new_line_length, line_count);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::None => return,
                },
                None => AppMode::Normal,
            },
        };

        self.state.app_mode = app_mode;
    }

    pub fn go_to_previous_half_page(&mut self) {
        if self.state.track.is_none() || self.state.subtitle_document.is_none() {
            self.switch_to_normal_mode();
        }

        let subtitle_document = match &self.state.subtitle_document {
            Some(document) => document,
            None => return,
        };

        let lines = self.renderer.get_lines_per_page() / 2;

        let app_mode = match &self.state.app_mode {
            AppMode::Normal => match self.get_first_active_cue_index() {
                Some(index) => AppMode::Select {
                    cue_index: index,
                    selected_cues: Vec::new(),
                },
                None => AppMode::Normal,
            },
            AppMode::Select {
                cue_index,
                selected_cues,
            } => AppMode::Select {
                cue_index: cue_index.saturating_sub(lines),
                selected_cues: selected_cues.clone(),
            },
            AppMode::Edit {
                cursor,
                selected_cues,
            } => {
                let new_cue_index = cursor.cue_index.saturating_sub(lines);
                let line_length = subtitle_document.cues.cue_len(new_cue_index);
                let new_cursor = Cursor::new(new_cue_index, line_length);

                AppMode::Edit {
                    cursor: new_cursor,
                    selected_cues: selected_cues.clone(),
                }
            }
        };

        self.state.app_mode = app_mode;
    }

    pub fn go_to_next_half_page(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        let lines = self.renderer.get_lines_per_page() / 2;
        let app_mode = match &self.state.app_mode {
            AppMode::Normal => match self.get_first_active_cue_index() {
                Some(index) => AppMode::Select {
                    cue_index: index,
                    selected_cues: Vec::new(),
                },
                None => AppMode::Normal,
            },
            AppMode::Select {
                cue_index,
                selected_cues,
            } => match &self.state.subtitle_document {
                Some(subtitle_document) => match &subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - lines {
                            index += lines;
                        } else {
                            index = cues.len() - 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - lines {
                            index += lines;
                        } else {
                            index = cues.len() - 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Line(cues) => {
                        let mut index = *cue_index;
                        if index < cues.len() - lines {
                            index += lines;
                        } else {
                            index = cues.len() - 1;
                        }

                        AppMode::Select {
                            cue_index: index,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::None => return,
                },
                None => AppMode::Normal,
            },
            AppMode::Edit {
                cursor,
                selected_cues,
            } => match &self.state.subtitle_document {
                Some(subtitle_document) => match &subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - lines {
                            new_index += lines;
                        } else {
                            new_index = cues.len() - 1;
                        }

                        let line_length = subtitle_document.cues.cue_len(new_index);
                        let new_cursor = Cursor::new(new_index, line_length);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - lines {
                            new_index += lines;
                        } else {
                            new_index = cues.len() - 1;
                        }

                        let line_length = subtitle_document.cues.cue_len(new_index);
                        let new_cursor = Cursor::new(new_index, line_length);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::Line(cues) => {
                        let mut new_index = cursor.cue_index;
                        if new_index < cues.len() - lines {
                            new_index += lines;
                        } else {
                            new_index = cues.len() - 1;
                        }

                        let line_length = subtitle_document.cues.cue_len(new_index);
                        let new_cursor = Cursor::new(new_index, line_length);

                        AppMode::Edit {
                            cursor: new_cursor,
                            selected_cues: selected_cues.clone(),
                        }
                    }
                    SubtitleCues::None => return,
                },
                None => AppMode::Normal,
            },
        };

        self.state.app_mode = app_mode;
    }
}
