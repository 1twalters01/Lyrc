use subtitles::subtitles::SubtitleCues;

use crate::{app::App, history::IndexedSubtitleCue, mode::AppMode, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub fn delete_selected_lines(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_documents.active_mut() {
            Some(subtitle_document_state) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for index in selected_cues.iter().rev() {
                            cues.remove(*index);
                            if index < cue_index {
                                *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                        *selected_cues = Vec::new();
                    }
                    SubtitleCues::Cue(cues) => {
                        for index in selected_cues.iter().rev() {
                            cues.remove(*index);
                            if index < cue_index {
                                *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                        *selected_cues = Vec::new();
                    }
                    SubtitleCues::Line(cues) => {
                        for index in selected_cues.iter().rev() {
                            cues.remove(*index);
                            if index < cue_index {
                                *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                        *selected_cues = Vec::new();
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    selected_cues,
                } => match &mut subtitle_document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for cue in selected_cues.iter().rev() {
                            cues.remove(cue.index);
                            if cue.index < cursor.cue_index {
                                let line_length = cues[cursor.cue_index.saturating_sub(1)]
                                    .words
                                    .iter()
                                    .map(|word| word.content.clone())
                                    .collect::<Vec<String>>()
                                    .join(" ")
                                    .len();
                                cursor.move_up(line_length);
                                // *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for cue in selected_cues.iter().rev() {
                            cues.remove(cue.index);
                            if cue.index < cursor.cue_index {
                                let line_length =
                                    cues[cursor.cue_index.saturating_sub(1)].content.len();
                                cursor.move_up(line_length)
                                // *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                    }
                    SubtitleCues::Line(cues) => {
                        for cue in selected_cues.iter().rev() {
                            cues.remove(cue.index);
                            if cue.index < cursor.cue_index {
                                let line_length =
                                    cues[cursor.cue_index.saturating_sub(1)].content.len();
                                cursor.move_up(line_length)
                                // *cue_index = cue_index.saturating_sub(1);
                            }
                        }
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn delete_current_line(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_documents.active_mut() {
            Some(subtitle_document_state) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        selected_cues.retain(|c_index| c_index != cue_index);
                        cues.remove(*cue_index);
                    }
                    SubtitleCues::Cue(cues) => {
                        selected_cues.retain(|c_index| c_index != cue_index);
                        cues.remove(*cue_index);
                    }
                    SubtitleCues::Line(cues) => {
                        selected_cues.retain(|c_index| c_index != cue_index);
                        cues.remove(*cue_index);
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    // cue_index,
                    selected_cues,
                } => match &mut subtitle_document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        selected_cues.retain(|cue| cue.index != cursor.cue_index);
                        cues.remove(cursor.cue_index);
                    }
                    SubtitleCues::Cue(cues) => {
                        selected_cues.retain(|cue| cue.index != cursor.cue_index);
                        cues.remove(cursor.cue_index);
                    }
                    SubtitleCues::Line(cues) => {
                        selected_cues.retain(|cue| cue.index != cursor.cue_index);
                        cues.remove(cursor.cue_index);
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn delete_cues(&mut self, mut indexed_cues: Vec<IndexedSubtitleCue>) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        indexed_cues.sort_by_key(|c| c.index);
        match &mut self.state.subtitle_documents.active_mut() {
            Some(subtitle_document_state) => match &mut subtitle_document_state.document.cues {
                SubtitleCues::Word(cues) => {
                    for indexed_cue in indexed_cues.iter().rev() {
                        cues.remove(indexed_cue.index);

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                selected_cues.retain(|c_index| *c_index != indexed_cue.index);
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue -= 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index -= 1;
                                    }
                                }
                            }
                        }
                    }
                }
                SubtitleCues::Cue(cues) => {
                    for indexed_cue in indexed_cues.iter().rev() {
                        cues.remove(indexed_cue.index);

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                selected_cues.retain(|c_index| *c_index != indexed_cue.index);
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue -= 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index -= 1;
                                    }
                                }
                            }
                        }
                    }
                }
                SubtitleCues::Line(cues) => {
                    for indexed_cue in indexed_cues.iter().rev() {
                        cues.remove(indexed_cue.index);

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                selected_cues.retain(|c_index| *c_index != indexed_cue.index);
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue -= 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index -= 1;
                                    }
                                }
                            }
                        }
                    }
                }
                SubtitleCues::None => {}
            },
            None => self.switch_to_normal_mode(),
        }
    }
}
