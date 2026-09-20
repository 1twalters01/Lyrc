use subtitles::subtitles::{AlignedCue, Cue, Line, SubtitleCues};
use uuid::Uuid;

use crate::{app::App, history::IndexedSubtitleCue, mode::AppMode, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub fn add_cue_after_current_cue(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_document {
            Some(subtitle_document) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        if cues.len() > *cue_index {
                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[*cue_index].start,
                                end: cues[*cue_index].end,
                                words: Vec::new(),
                            };
                            cues.insert(*cue_index + 1, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if *cue + 1 >= *cue_index {
                                    *cue += 1;
                                }
                            }
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        if cues.len() > *cue_index {
                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[*cue_index].start,
                                end: cues[*cue_index].end,
                                content: String::new(),
                            };
                            cues.insert(*cue_index + 1, empty_subtitle);
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        if lines.len() > *cue_index {
                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };
                            lines.insert(*cue_index + 1, empty_subtitle);
                        }
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        if cues.len() > cursor.cue_index {
                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[cursor.cue_index].start,
                                end: cues[cursor.cue_index].end,
                                words: Vec::new(),
                            };
                            cues.insert(cursor.cue_index + 1, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index + 1 >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        if cues.len() > cursor.cue_index {
                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[cursor.cue_index].start,
                                end: cues[cursor.cue_index].end,
                                content: String::new(),
                            };
                            cues.insert(cursor.cue_index + 1, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index + 1 >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        if lines.len() > cursor.cue_index {
                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };
                            lines.insert(cursor.cue_index + 1, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index + 1 >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                        }
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn add_cue_before_current_cue(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_document {
            Some(subtitle_document) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        if cues.len() > *cue_index {
                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[*cue_index].start,
                                end: cues[*cue_index].end,
                                words: Vec::new(),
                            };
                            cues.insert(*cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue >= cue_index {
                                    *cue += 1;
                                }
                            }
                            *cue_index += 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        if cues.len() > *cue_index {
                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[*cue_index].start,
                                end: cues[*cue_index].end,
                                content: String::new(),
                            };
                            cues.insert(*cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue >= cue_index {
                                    *cue += 1;
                                }
                            }
                            *cue_index += 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        if lines.len() > *cue_index {
                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };
                            lines.insert(*cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue >= cue_index {
                                    *cue += 1;
                                }
                            }
                            *cue_index += 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        if cues.len() > cursor.cue_index {
                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[cursor.cue_index].start,
                                end: cues[cursor.cue_index].end,
                                words: Vec::new(),
                            };
                            cues.insert(cursor.cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                            let line_length = cues[cursor.cue_index + 1]
                                .words
                                .iter()
                                .map(|word| word.content.clone())
                                .collect::<Vec<String>>()
                                .join(" ")
                                .len();
                            cursor.move_down(line_length, cues.len())
                            // *cue_index += 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        if cues.len() > cursor.cue_index {
                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[cursor.cue_index].start,
                                end: cues[cursor.cue_index].end,
                                content: String::new(),
                            };
                            cues.insert(cursor.cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                            cursor.cue_index += 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        if lines.len() > cursor.cue_index {
                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };
                            lines.insert(cursor.cue_index, empty_subtitle);

                            for cue in selected_cues.iter_mut() {
                                if cue.index >= cursor.cue_index {
                                    cue.index += 1;
                                }
                            }
                            cursor.cue_index += 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn add_cue_after_selected_cues(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_document {
            Some(subtitle_document) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[*cue_index].end,
                                words: Vec::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[*cue_index].end,
                                content: String::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };

                            lines.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[cursor.cue_index].end,
                                words: Vec::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[cursor.cue_index].end,
                                content: String::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };

                            lines.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn add_cue_before_selected_cues(&mut self) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        match &mut self.state.subtitle_document {
            Some(subtitle_document) => match &mut self.state.app_mode {
                AppMode::Normal => {}
                AppMode::Select {
                    cue_index,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[*cue_index].end,
                                words: Vec::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[*cue_index].end,
                                content: String::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i] + i;

                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };

                            lines.insert(index, empty_subtitle);

                            selected_cues[i] += i + 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
                AppMode::Edit {
                    cursor,
                    selected_cues,
                } => match &mut subtitle_document.cues {
                    SubtitleCues::Word(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = AlignedCue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[cursor.cue_index].end,
                                words: Vec::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = Cue {
                                id: Uuid::new_v4(),
                                start: cues[index].start,
                                end: cues[cursor.cue_index].end,
                                content: String::new(),
                            };

                            cues.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::Line(lines) => {
                        for i in 0..selected_cues.len() {
                            let index = selected_cues[i].index + i;

                            let empty_subtitle = Line {
                                id: Uuid::new_v4(),
                                content: String::new(),
                            };

                            lines.insert(index, empty_subtitle);

                            selected_cues[i].index += i + 1;
                        }
                    }
                    SubtitleCues::None => {}
                },
            },
            None => self.switch_to_normal_mode(),
        }
    }

    pub fn insert_cues(&mut self, mut indexed_cues: Vec<IndexedSubtitleCue>) {
        if self.state.track.is_none() {
            self.switch_to_normal_mode();
        }

        indexed_cues.sort_by_key(|c| c.index);
        match &mut self.state.subtitle_document {
            Some(subtitle_document) => match &mut subtitle_document.cues {
                SubtitleCues::Word(cues) => {
                    for indexed_cue in indexed_cues {
                        match indexed_cue.subtitle_cue {
                            SubtitleCues::Word(cue) => {
                                cues.insert(indexed_cue.index, cue[0].clone())
                            }
                            _ => {}
                        }

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue += 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                SubtitleCues::Cue(cues) => {
                    for indexed_cue in indexed_cues {
                        match indexed_cue.subtitle_cue {
                            SubtitleCues::Cue(cue) => {
                                cues.insert(indexed_cue.index, cue[0].clone())
                            }
                            _ => {}
                        }

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue += 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                SubtitleCues::Line(lines) => {
                    for indexed_cue in indexed_cues {
                        match indexed_cue.subtitle_cue {
                            SubtitleCues::Line(cue) => {
                                lines.insert(indexed_cue.index, cue[0].clone())
                            }
                            _ => {}
                        }

                        match &mut self.state.app_mode {
                            AppMode::Normal => {}
                            AppMode::Select {
                                cue_index: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if *selected_cue > indexed_cue.index {
                                        *selected_cue += 1;
                                    }
                                }
                            }
                            AppMode::Edit {
                                cursor: _,
                                selected_cues,
                            } => {
                                for selected_cue in selected_cues {
                                    if selected_cue.index > indexed_cue.index {
                                        selected_cue.index += 1;
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
