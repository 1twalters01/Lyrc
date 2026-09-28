use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueContentChange, Edit},
    mode::AppMode,
    renderer::Renderer,
};
use subtitles::subtitles::SubtitleCues;

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    _config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut document_state = app.state.subtitle_documents.active_mut();
    let document_state = match &mut document_state {
        Some(document_state) => document_state,
        None => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

        let (cursor, selected_cues) = match &mut app.state.app_mode {
            AppMode::Edit { cursor, selected_cues } => (cursor, selected_cues),
            _ => {
                app.switch_to_normal_mode();
                return Ok(());
            }
    };

    match key.code {
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,

        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
            document_state.document.save()?;
            document_state.unsaved_changes = false;
            app.state.reload_subtitle_documents().await;
            if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                app.state.subtitle_documents.set_variant(active_variant);
            }
        }

        // Undo and redo changes
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => app.undo(),
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => app.redo(),

        KeyCode::Esc => {
            if document_state.unsaved_changes == true {
                app.state.reload_subtitle_documents().await;
                if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                    app.state.subtitle_documents.set_variant(active_variant);
                }
            } else {
                app.switch_to_select_mode()?
            }
        }

        KeyCode::Tab => app.switch_to_normal_mode(),
        KeyCode::Enter => app.switch_to_select_mode()?,

        KeyCode::Left => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.move_left(),
            _ => {}
        },
        KeyCode::Right => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.move_right(document_state.document.cues.cue_len(cursor.cue_index)),
            _ => {}
        },
        KeyCode::Up => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.move_up(
                document_state
                    .document
                    .cues
                    .cue_len(cursor.cue_index.saturating_sub(1)),
            ),
            _ => {}
        },
        KeyCode::Down => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => {
                let line_count = document_state.document.cues.len();
                let new_line_index = std::cmp::min(
                    cursor.cue_index.saturating_add(1),
                    document_state.document.cues.len().saturating_sub(1),
                );
                let new_line_length = document_state.document.cues.cue_len(new_line_index);
                cursor.move_down(new_line_length, line_count)
            }
            _ => {}
        },

        KeyCode::Char(char) => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                document_state.unsaved_changes = false;

                let mut column = 0;
                let mut insertion = None;

                for (word_idx, word) in current_cue.words.iter().enumerate() {
                    let word_len = word.content.len();

                    if cursor.active_column <= column + word_len {
                        insertion = Some((word_idx, cursor.active_column - column));
                        break
                    }

                    column += word_len;

                    if word_idx + 1 < current_cue.words.len() {
                        column += 1;
                    }
                }

                if let Some((word_idx, insert_idx)) = insertion {
                    current_cue.words[word_idx].content.insert(insert_idx, char);

                    cursor.active_column += 1;
                    cursor.target_column = cursor.active_column;
                }
                                
                let new_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            },
            SubtitleCues::Cue(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                document_state.unsaved_changes = false;

                let insert_idx = std::cmp::min(
                    cursor.active_column,
                    current_cue.content.len(),
                );
                current_cue.content.insert(insert_idx, char);
                cursor.active_column += 1;
                cursor.target_column = cursor.active_column;
                
                let new_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            },
            SubtitleCues::Line(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                document_state.unsaved_changes = false;

                let insert_idx = std::cmp::min(
                    cursor.active_column,
                    current_cue.content.len(),
                );
                current_cue.content.insert(insert_idx, char);
                cursor.active_column += 1;
                cursor.target_column = cursor.active_column;

                let new_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            },
            SubtitleCues::None => {},
        },
        KeyCode::Backspace => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                if cursor.active_column == 0 || current_cue.words.is_empty() {
                    return Ok(());
                }

                document_state.unsaved_changes = false;

                let mut column = 0;
                let mut deletion = None;
                let delete_column = cursor.active_column.saturating_sub(1);

                for (word_idx, word) in current_cue.words.iter().enumerate() {
                    let word_len = word.content.len();

                    if delete_column < column + word_len {
                        let delete_idx = delete_column - column;
                        deletion = Some((word_idx, delete_idx));
                        break;
                    }

                    column += word_len;

                    if word_idx + 1 < current_cue.words.len() {
                        // delete_column is on the space between this word and the next
                        if delete_column == column {
                            // Decide what backspace should do here.
                            //
                            // For example, merge the next word into this word.
                            deletion = Some((word_idx, word_len));
                            break;
                        }

                        column += 1;
                    }
                }

                if let Some((word_idx, delete_idx)) = deletion {
                    current_cue.words[word_idx].content.remove(delete_idx);

                    cursor.active_column = cursor.active_column.saturating_sub(1);
                    cursor.target_column = cursor.active_column;
                }

                let new_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                cursor.active_column = cursor.active_column.min(current_cue.content.len());

                if cursor.active_column == 0 || current_cue.content.is_empty() {
                    return Ok(());
                }

                document_state.unsaved_changes = false;

                let delete_idx = cursor.active_column.saturating_sub(1);
                current_cue.content.remove(delete_idx);
                cursor.active_column = cursor.active_column.saturating_sub(1);
                cursor.target_column = cursor.active_column;

                let new_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                cursor.active_column = cursor.active_column.min(current_cue.content.len());

                if cursor.active_column == 0 || current_cue.content.is_empty() {
                    return Ok(());
                }

                document_state.unsaved_changes = false;

                let delete_idx = cursor.active_column.saturating_sub(1);
                current_cue.content.remove(delete_idx);
                cursor.active_column = cursor.active_column.saturating_sub(1);
                cursor.target_column = cursor.active_column;

                let new_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                let edit = Edit::EditCueContent {
                    changes: Vec::from([CueContentChange {
                        index: cursor.cue_index,
                        old_content,
                        new_content,
                    }]),
                };
                app.push_to_history(edit);
            }
            SubtitleCues::None => {}
        },

        _ => {}
    }

    Ok(())
}
