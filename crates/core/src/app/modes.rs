use subtitles::subtitles::SubtitleCues;

use crate::{
    app::App,
    mode::{AppMode, Cursor, EditCue},
    renderer::Renderer,
};

impl<R> App<R>
where
    R: Renderer,
{
    pub fn switch_to_normal_mode(&mut self) {
        self.state.app_mode = AppMode::Normal;
    }

    pub fn switch_to_select_mode(&mut self) -> Result<(), String> {
        if self.state.subtitle_documents.active().is_none() {
            return Err(String::from("No subtitle document found"));
        }

        let (cue_index, selected_cues) = match &self.state.app_mode {
            AppMode::Normal => (
                *self
                    .synchronizer
                    .get_active_indices()
                    .iter()
                    .map(|i| i.cue_index().cue)
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap_or(&0),
                Vec::new(),
            ),
            AppMode::Select {
                cue_index,
                selected_cues,
            } => (*cue_index, selected_cues.clone()),
            AppMode::Edit {
                cursor,
                selected_cues,
            } => (
                cursor.cue_index,
                selected_cues
                    .iter()
                    .map(|edit_cue| edit_cue.index)
                    .collect(),
            ),
        };

        self.state.app_mode = AppMode::Select {
            cue_index,
            selected_cues,
        };

        Ok(())
    }

    pub fn switch_to_edit_mode(&mut self) -> Result<(), String> {
        let subtitle_document_state = &self.state.subtitle_documents.active();
        let subtitle_document_state = match &subtitle_document_state {
            Some(subtitle_document_state) => subtitle_document_state,
            None => return Err(String::from("No subtitle document found")),
        };

        let (cursor, selected_cues) = match &self.state.app_mode {
            AppMode::Normal => {
                let index = *self
                    .synchronizer
                    .get_active_indices()
                    .iter()
                    .map(|i| i.cue_index().cue)
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap_or(&0);

                let original_content = match &subtitle_document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        SubtitleCues::Word(Vec::from([cues[index].clone()]))
                    }
                    SubtitleCues::Cue(cues) => SubtitleCues::Cue(Vec::from([cues[index].clone()])),
                    SubtitleCues::Line(lines) => {
                        SubtitleCues::Line(Vec::from([lines[index].clone()]))
                    }
                    SubtitleCues::None => SubtitleCues::None,
                };

                let selected_edit_cues = Vec::from([EditCue {
                    index,
                    original_content,
                }]);
                let line_length = &subtitle_document_state.document.cues.cue_len(index);
                let cursor = Cursor::new(index, *line_length);
                (cursor, selected_edit_cues.clone())
            }
            AppMode::Select {
                cue_index,
                selected_cues,
            } => {
                let selected_edit_cues = selected_cues
                    .iter()
                    .map(|index| EditCue {
                        index: *index,
                        original_content: match &subtitle_document_state.document.cues {
                            SubtitleCues::Word(cues) => {
                                SubtitleCues::Word(Vec::from([cues[*index].clone()]))
                            }
                            SubtitleCues::Cue(cues) => {
                                SubtitleCues::Cue(Vec::from([cues[*index].clone()]))
                            }
                            SubtitleCues::Line(lines) => {
                                SubtitleCues::Line(Vec::from([lines[*index].clone()]))
                            }
                            SubtitleCues::None => SubtitleCues::None,
                        },
                    })
                    .collect::<Vec<EditCue>>();
                let line_length = &subtitle_document_state.document.cues.cue_len(*cue_index);
                let cursor = Cursor::new(*cue_index, *line_length);
                (cursor, selected_edit_cues.clone())
            }

            AppMode::Edit {
                cursor,
                selected_cues,
            } => (cursor.clone(), selected_cues.clone()),
        };

        self.state.app_mode = AppMode::Edit {
            cursor,
            selected_cues,
        };

        Ok(())
    }
}
