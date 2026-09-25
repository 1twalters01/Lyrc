use subtitles::subtitles::SubtitleCues;

use crate::{
    app::App,
    history::{CueTimeChange, Edit},
    renderer::Renderer,
};

impl<R> App<R>
where
    R: Renderer,
{
    pub fn push_to_history(&mut self, edit: Edit) -> bool {
        if let Some(document_state) = self.state.subtitle_documents.active_mut() {
            document_state.edit_history.push(edit);
            return true;
        }

        false
    }

    pub fn undo(&mut self) {
        let Some(edit) = ({
            let Some(document_state) = self.state.subtitle_documents.active_mut() else {
                return;
            };

            document_state.edit_history.pop_undo()
        }) else {
            return;
        };

        self.undo_edit(&edit);

        if let Some(document_state) = self.state.subtitle_documents.active_mut() {
            document_state.edit_history.push_redo(edit);

            if document_state.edit_history.is_empty() {
                document_state.unsaved_changes = false;
            }
        }
    }

    pub fn redo(&mut self) {
        let Some(edit) = ({
            let Some(document_state) = self.state.subtitle_documents.active_mut() else {
                return;
            };

            document_state.edit_history.pop_redo()
        }) else {
            return;
        };

        self.redo_edit(&edit);

        if let Some(document_state) = self.state.subtitle_documents.active_mut() {
            document_state.unsaved_changes = true;
            document_state.edit_history.push_undo(edit);
        }
    }

    fn undo_edit(&mut self, edit: &Edit) {
        match &mut self.state.subtitle_documents.active_mut() {
            Some(subtitle_document_state) => match edit {
                Edit::EditCueContent { changes } => {
                    match &mut subtitle_document_state.document.cues {
                        SubtitleCues::Word(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Word(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::Cue(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Cue(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::Line(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Line(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::None => {}
                    }
                }
                Edit::EditCueTimes { changes } => {
                    let inverse_changes = changes
                        .iter()
                        .map(|change| CueTimeChange {
                            id: change.id.clone(),
                            new_index: change.old_index,
                            old_index: change.new_index,
                            new_start: change.old_start,
                            old_start: change.new_start,
                            new_end: change.old_end,
                            old_end: change.new_end,
                        })
                        .collect::<Vec<CueTimeChange>>();
                    self.set_times(inverse_changes.clone());
                }
                Edit::DeleteCue { cues } => {
                    self.insert_cues(cues.clone());
                }
                Edit::InsertCue { cues } => {
                    self.delete_cues(cues.clone());
                }
            },
            None => {}
        }
    }

    fn redo_edit(&mut self, edit: &Edit) {
        match &mut self.state.subtitle_documents.active_mut() {
            Some(subtitle_document_state) => match edit {
                Edit::EditCueContent { changes } => {
                    match &mut subtitle_document_state.document.cues {
                        SubtitleCues::Word(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Word(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::Cue(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Cue(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::Line(subtitle_cues) => {
                            for change in changes {
                                if let SubtitleCues::Line(cues) = &change.old_content {
                                    subtitle_cues[change.index] = cues[0].clone();
                                }
                            }
                        }
                        SubtitleCues::None => {}
                    }
                }
                Edit::EditCueTimes { changes } => {
                    self.set_times(changes.clone());
                }
                Edit::DeleteCue { cues } => {
                    self.delete_cues(cues.clone());
                }
                Edit::InsertCue { cues } => {
                    self.insert_cues(cues.clone());
                }
            },
            None => {}
        }
    }
}
