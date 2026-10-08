use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueContentChange, Edit},
    mode::AppMode,
    renderer::Renderer,
    repositories::subtitle_document::SubtitleDocumentRepository,
};
use subtitles::subtitles::{SubtitleCues, Word};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    _config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let subtitle_variant = app.state.subtitle_documents.active_variant();

    let mut document_state = app.state.subtitle_documents.active_mut();
    let document_state = match &mut document_state {
        Some(document_state) => document_state,
        None => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

    let (cursor, selected_cues) = match &mut app.state.app_mode {
        AppMode::Edit {
            cursor,
            selected_cues,
        } => (cursor, selected_cues),
        _ => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

    match key.code {
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,

        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
            document_state.document.save()?;

            let pool = app.database_service.get_pool();
            let subtitle_document_repository = SubtitleDocumentRepository::new(&pool);

            if let Some(track) = &app.state.track
                && let Some(variant) = subtitle_variant
                && document_state.document.metadata.title.is_some()
                && !document_state.document.metadata.artists.is_empty()
                && !document_state.document.metadata.languages.is_empty()
            {
                subtitle_document_repository
                    .save(&document_state.document, track.duration, variant)
                    .await?;
            }

            document_state.unsaved_changes = false;
            app.state.reload_subtitle_documents().await;
            if let (Some(active_variant), Some(state)) = (
                app.state.subtitle_documents.active_variant(),
                app.state.subtitle_documents.active(),
            ) {
                app.state
                    .subtitle_documents
                    .insert_cache_forced(active_variant.clone(), state.clone());
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
                    if let Some(state) =
                        app.state.subtitle_documents.get_from_cache(&active_variant)
                    {
                        app.state
                            .subtitle_documents
                            .insert_forced(active_variant.clone(), state.clone());
                    }
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
        KeyCode::End => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.end(document_state.document.cues.cue_len(cursor.cue_index)),
            _ => {}
        },
        KeyCode::Home => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.home(),
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

                let mut column = 0;

                if current_cue.words.is_empty() {
                    if char == ' ' {
                        current_cue.words.push(Word {
                            content: String::new(),
                            start: current_cue.start,
                            end: current_cue.end,
                        })
                    } else {
                        current_cue.words.push(Word {
                            content: char.to_string(),
                            start: current_cue.start,
                            end: current_cue.end,
                        })
                    }

                    document_state.unsaved_changes = true;

                    cursor.active_column = 1;
                    cursor.target_column = cursor.active_column;
                } else {
                    let current_cue_word_count = current_cue.words.len();
                    for (word_idx, word) in current_cue.words.iter_mut().enumerate() {
                        let word_len = word.content.chars().count();

                        if cursor.active_column <= column + word_len {
                            let char_idx = cursor.active_column - column;

                            let byte_idx = word
                                .content
                                .char_indices()
                                .nth(char_idx)
                                .map(|(idx, _)| idx)
                                .unwrap_or(word.content.len());

                            if char == ' ' {
                                let mut new_word = word.clone();
                                new_word.content = word.content.split_off(byte_idx);

                                current_cue.words.insert(word_idx + 1, new_word);
                            } else {
                                current_cue.words[word_idx].content.insert(byte_idx, char);
                            }

                            document_state.unsaved_changes = true;

                            cursor.active_column += 1;
                            cursor.target_column = cursor.active_column;

                            break;
                        }

                        column += word_len;

                        if word_idx + 1 < current_cue_word_count {
                            column += 1;
                        }
                    }
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

                let char_len = current_cue.content.chars().count();
                let insert_idx = std::cmp::min(cursor.active_column, char_len);
                let byte_idx = current_cue
                    .content
                    .char_indices()
                    .nth(insert_idx)
                    .map(|(idx, _)| idx)
                    .unwrap_or(current_cue.content.len());

                current_cue.content.insert(byte_idx, char);
                document_state.unsaved_changes = true;
                cursor.active_column = insert_idx + 1;
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

                let char_len = current_cue.content.chars().count();
                let insert_idx = std::cmp::min(cursor.active_column, char_len);
                let byte_idx = current_cue
                    .content
                    .char_indices()
                    .nth(insert_idx)
                    .map(|(idx, _)| idx)
                    .unwrap_or(current_cue.content.len());

                current_cue.content.insert(byte_idx, char);
                document_state.unsaved_changes = true;
                cursor.active_column = insert_idx + 1;
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
        KeyCode::Backspace => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                if cursor.active_column == 0 || current_cue.words.is_empty() {
                    return Ok(());
                }

                let mut column = 0;
                let delete_column = cursor.active_column - 1;

                for word_idx in 0..current_cue.words.len() {
                    let word_len = current_cue.words[word_idx].content.chars().count();

                    // Backspace at the first character of a word to merge them
                    if delete_column == column && word_idx > 0 {
                        let current_content = current_cue.words[word_idx].content.clone();

                        current_cue.words[word_idx - 1]
                            .content
                            .push_str(&current_content);

                        current_cue.words.remove(word_idx);

                        document_state.unsaved_changes = true;

                        cursor.active_column = column - 1;
                        cursor.target_column = cursor.active_column;

                        let new_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                        let edit = Edit::EditCueContent {
                            changes: Vec::from([CueContentChange {
                                index: cursor.cue_index,
                                old_content,
                                new_content,
                            }]),
                        };

                        app.push_to_history(edit);

                        break;
                    }

                    // Normal character deletion.
                    if delete_column < column + word_len {
                        let char_idx = delete_column - column;

                        if let Some((byte_idx, _)) = current_cue.words[word_idx]
                            .content
                            .char_indices()
                            .nth(char_idx)
                        {
                            current_cue.words[word_idx].content.remove(byte_idx);

                            document_state.unsaved_changes = true;

                            cursor.active_column -= 1;
                            cursor.target_column = cursor.active_column;

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

                        break;
                    }

                    column += word_len;

                    // Account for the space between words.
                    if word_idx + 1 < current_cue.words.len() {
                        column += 1;
                    }
                }
            }
            SubtitleCues::Cue(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                let char_len = current_cue.content.chars().count();
                cursor.active_column = cursor.active_column.min(char_len);

                if cursor.active_column == 0 || current_cue.content.is_empty() {
                    return Ok(());
                }

                let char_index = cursor.active_column.saturating_sub(1);
                if let Some((byte_index, _)) = current_cue.content.char_indices().nth(char_index) {
                    current_cue.content.remove(byte_index);
                    document_state.unsaved_changes = true;

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
            }
            SubtitleCues::Line(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                let char_len = current_cue.content.chars().count();
                cursor.active_column = cursor.active_column.min(char_len);

                if cursor.active_column == 0 || current_cue.content.is_empty() {
                    return Ok(());
                }

                let char_index = cursor.active_column.saturating_sub(1);
                if let Some((byte_index, _)) = current_cue.content.char_indices().nth(char_index) {
                    current_cue.content.remove(byte_index);
                    document_state.unsaved_changes = true;

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
            }
            SubtitleCues::None => {}
        },
        KeyCode::Delete => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                if current_cue.words.is_empty() {
                    return Ok(());
                }

                let mut column = 0;
                let delete_column = cursor.active_column;

                for word_idx in 0..current_cue.words.len() {
                    let word_len = current_cue.words[word_idx].content.chars().count();

                    // Delete at the last character of a word to merge them
                    if delete_column == column + word_len && word_idx + 1 < current_cue.words.len()
                    {
                        let next_content = current_cue.words[word_idx + 1].content.clone();

                        current_cue.words[word_idx].content.push_str(&next_content);

                        current_cue.words.remove(word_idx + 1);

                        document_state.unsaved_changes = true;

                        let new_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                        let edit = Edit::EditCueContent {
                            changes: Vec::from([CueContentChange {
                                index: cursor.cue_index,
                                old_content,
                                new_content,
                            }]),
                        };

                        app.push_to_history(edit);

                        break;
                    }

                    // Normal character deletion.
                    if delete_column < column + word_len {
                        let char_idx = delete_column - column;

                        if let Some((byte_idx, _)) = current_cue.words[word_idx]
                            .content
                            .char_indices()
                            .nth(char_idx)
                        {
                            current_cue.words[word_idx].content.remove(byte_idx);

                            document_state.unsaved_changes = true;

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

                        break;
                    }

                    column += word_len;

                    // Account for the space between words.
                    if word_idx + 1 < current_cue.words.len() {
                        column += 1;
                    }
                }
            }
            SubtitleCues::Cue(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Cue(Vec::from([current_cue.clone()]));

                let char_len = current_cue.content.chars().count();
                cursor.active_column = cursor.active_column.min(char_len);

                if cursor.active_column == char_len || current_cue.content.is_empty() {
                    return Ok(());
                }

                let char_index = cursor.active_column;
                if let Some((byte_index, _)) = current_cue.content.char_indices().nth(char_index) {
                    current_cue.content.remove(byte_index);
                    document_state.unsaved_changes = true;

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
            }
            SubtitleCues::Line(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Line(Vec::from([current_cue.clone()]));

                let char_len = current_cue.content.chars().count();
                cursor.active_column = cursor.active_column.min(char_len);

                if cursor.active_column == char_len || current_cue.content.is_empty() {
                    return Ok(());
                }

                let char_index = cursor.active_column;
                if let Some((byte_index, _)) = current_cue.content.char_indices().nth(char_index) {
                    current_cue.content.remove(byte_index);
                    document_state.unsaved_changes = true;

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
            }
            SubtitleCues::None => {}
        },

        _ => {}
    }

    Ok(())
}
