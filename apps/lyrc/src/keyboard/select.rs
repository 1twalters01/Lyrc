use core::option::Option::None;

use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueTimeChange, Edit, IndexedSubtitleCue},
    mode::AppMode,
    renderer::Renderer,
    repositories::subtitle_document::SubtitleDocumentRepository,
};
use subtitles::subtitles::{SubtitleCues, SyncLevel};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    config: &Config,
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

    let (cursor, selected_cues) = match &app.state.app_mode {
        AppMode::Select {
            cursor,
            selected_cues,
        } => (cursor, selected_cues),
        _ => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

    match key.code {
        // Quit
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
            app.state.quit = true;
        }

        // Save
        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
            document_state.document.save()?;

            let pool = app.database_service.get_pool();
            let subtitle_document_repository = SubtitleDocumentRepository::new(&pool);

            if let Some(track) = &app.state.track
                && let Some(variant) = subtitle_variant
                && document_state.document.metadata.title.is_some()
                && !document_state.document.metadata.artists.is_empty()
            {
                subtitle_document_repository
                    .save(&document_state.document, track.duration, variant)
                    .await?;
            }

            document_state.unsaved_changes = false;
            app.state.reload_subtitle_documents().await;
            if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                app.state.subtitle_documents.set_variant(active_variant);
            }
        }

        // Undo and redo changes
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => app.undo(),
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => app.redo(),

        // Mode change
        KeyCode::Esc => {
            if document_state.unsaved_changes == true {
                app.state.reload_subtitle_documents().await;
                if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                    if let Some(state) =
                        app.state.subtitle_documents.get_from_cache(&active_variant)
                    {
                        println!("state: {:?}", state.unsaved_changes);
                        app.state
                            .subtitle_documents
                            .insert_forced(active_variant.clone(), state.clone());
                    }
                    app.state.subtitle_documents.set_variant(active_variant);
                }
            } else {
                app.switch_to_normal_mode()
            }
        }
        KeyCode::Tab => app.switch_to_edit_mode()?,
        KeyCode::Enter => match cursor.active_word_index {
            Some(active_word_index) => {
                app.seek_to_selected_word(cursor.cue_index, active_word_index)
                    .await?
            }
            None => app.seek_to_selected_line(cursor.cue_index).await?,
        },

        // Line control
        KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => app.go_to_previous_half_page(),
        KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => app.go_to_next_half_page(),
        KeyCode::Up => app.go_to_previous_line(),
        KeyCode::Down => app.go_to_next_line(),
        KeyCode::Left if key.modifiers == KeyModifiers::CONTROL => match &mut app.state.app_mode {
            AppMode::Select {
                cursor,
                selected_cues: _,
            } => {
                if cursor.active_word_index == Some(0) {
                    cursor.active_word_index = None
                } else {
                    cursor.move_left()
                }
            }
            _ => {}
        },
        KeyCode::Right if key.modifiers == KeyModifiers::CONTROL => match &mut app.state.app_mode {
            AppMode::Select {
                cursor,
                selected_cues: _,
            } => {
                if document_state.document.sync_level() >= SyncLevel::Word
                    && cursor.active_word_index == None
                {
                    cursor.active_word_index = Some(0);
                } else {
                    cursor.move_right(document_state.document.cues.word_count(cursor.cue_index))
                }
            }
            _ => {}
        },
        KeyCode::End => match &mut app.state.app_mode {
            AppMode::Select {
                cursor,
                selected_cues: _,
            } => {
                if document_state.document.sync_level() >= SyncLevel::Word {
                    cursor.end(document_state.document.cues.word_count(cursor.cue_index))
                }
            }
            _ => {}
        },
        KeyCode::Home => match &mut app.state.app_mode {
            AppMode::Select {
                cursor,
                selected_cues: _,
            } => {
                if document_state.document.sync_level() >= SyncLevel::Word {
                    cursor.home()
                }
            }
            _ => {}
        },

        KeyCode::Char('H') => app.toggle_select_all_lines()?,
        KeyCode::Char('h') => app.toggle_select_line(),

        // Playback control
        KeyCode::Char(' ') => app.toggle_play_pause().await?,
        KeyCode::Left => app.seek_by_duration(config.rewind_duration).await?,
        KeyCode::Right => app.seek_by_duration(config.fast_forward_duration).await?,

        // Edit cues
        KeyCode::Char('D') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cues = match &app.state.app_mode {
                    AppMode::Normal => Vec::new(),
                    AppMode::Select {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: *cue,
                            subtitle_cue: SubtitleCues::Word(Vec::from([cues[*cue].clone()])),
                        })
                        .collect(),
                    AppMode::Edit {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: cue.index,
                            subtitle_cue: SubtitleCues::Word(Vec::from([cues[cue.index].clone()])),
                        })
                        .collect(),
                };

                document_state.unsaved_changes = true;
                app.delete_selected_lines();
                let edit = Edit::DeleteCue { cues };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cues = match &app.state.app_mode {
                    AppMode::Normal => Vec::new(),
                    AppMode::Select {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: *cue,
                            subtitle_cue: SubtitleCues::Cue(Vec::from([cues[*cue].clone()])),
                        })
                        .collect(),
                    AppMode::Edit {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: cue.index,
                            subtitle_cue: SubtitleCues::Cue(Vec::from([cues[cue.index].clone()])),
                        })
                        .collect(),
                };

                document_state.unsaved_changes = true;
                app.delete_selected_lines();
                let edit = Edit::DeleteCue { cues };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(cues) => {
                let cues = match &app.state.app_mode {
                    AppMode::Normal => Vec::new(),
                    AppMode::Select {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: *cue,
                            subtitle_cue: SubtitleCues::Line(Vec::from([cues[*cue].clone()])),
                        })
                        .collect(),
                    AppMode::Edit {
                        cursor: _,
                        selected_cues,
                    } => selected_cues
                        .iter()
                        .map(|cue| IndexedSubtitleCue {
                            index: cue.index,
                            subtitle_cue: SubtitleCues::Line(Vec::from([cues[cue.index].clone()])),
                        })
                        .collect(),
                };

                document_state.unsaved_changes = true;
                app.delete_selected_lines();
                let edit = Edit::DeleteCue { cues };
                app.push_to_history(edit);
            }
            SubtitleCues::None => {}
        },
        KeyCode::Char('d') => app.delete_current_line(),

        KeyCode::Char('k') => app.add_cue_before_current_cue(),
        KeyCode::Char('j') => app.add_cue_after_current_cue(),
        KeyCode::Char('o') => app.add_cue_before_selected_cues(),
        KeyCode::Char('i') => app.add_cue_after_selected_cues(),

        // Adjust cue time
        KeyCode::Char('m') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_start_time(config.backwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_start_time(config.backwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char(',') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_start_time(config.forwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_start_time(config.forwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('.') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_end_time(config.backwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_end_time(config.backwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('/') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_end_time(config.forwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_end_time(config.forwards_cue_increment_small)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('M') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_start_time(config.backwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_start_time(config.backwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('<') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_start_time(config.forwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_start_time(config.forwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('>') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_end_time(config.backwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.decrease_current_cue_end_time(config.backwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('?') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_end_time(config.forwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Cue(cues) => {
                let cue = &cues[cursor.cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cursor.cue_index,
                    old_index: cursor.cue_index,
                    new_start: cue.start,
                    old_start: cue.start,
                    new_end: cue.end,
                    old_end: cue.end,
                }]);

                for change in &mut changes {
                    if let Some((i, cue)) =
                        cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                    {
                        change.new_index = i;
                        change.new_start = cue.start;
                        change.new_end = cue.end;
                    };
                }

                document_state.unsaved_changes = true;
                app.increase_current_cue_end_time(config.forwards_cue_increment_large)?;

                let edit = Edit::EditCueTimes { changes };
                app.push_to_history(edit);
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },
        KeyCode::Char('c') => match app.clock.get_position() {
            Some(position) => match &mut document_state.document.cues {
                SubtitleCues::Word(cues) => {
                    let cue = &cues[cursor.cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cursor.cue_index,
                        old_index: cursor.cue_index,
                        new_start: cue.start,
                        old_start: cue.start,
                        new_end: cue.end,
                        old_end: cue.end,
                    }]);

                    for change in &mut changes {
                        if let Some((i, cue)) =
                            cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                        {
                            change.new_index = i;
                            change.new_start = cue.start;
                            change.new_end = cue.end;
                        };
                    }

                    document_state.unsaved_changes = true;
                    app.set_current_cue_start_time(position)?;

                    let edit = Edit::EditCueTimes { changes };
                    app.push_to_history(edit);
                }
                SubtitleCues::Cue(cues) => {
                    let cue = &cues[cursor.cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cursor.cue_index,
                        old_index: cursor.cue_index,
                        new_start: cue.start,
                        old_start: cue.start,
                        new_end: cue.end,
                        old_end: cue.end,
                    }]);

                    for change in &mut changes {
                        if let Some((i, cue)) =
                            cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                        {
                            change.new_index = i;
                            change.new_start = cue.start;
                            change.new_end = cue.end;
                        };
                    }

                    document_state.unsaved_changes = true;
                    app.set_current_cue_start_time(position)?;

                    let edit = Edit::EditCueTimes { changes };
                    app.push_to_history(edit);
                }
                SubtitleCues::Line(_) => {}
                SubtitleCues::None => {}
            },
            None => {}
        },
        KeyCode::Char('C') => match app.clock.get_position() {
            Some(position) => match &mut document_state.document.cues {
                SubtitleCues::Word(cues) => {
                    let cue = &cues[cursor.cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cursor.cue_index,
                        old_index: cursor.cue_index,
                        new_start: cue.start,
                        old_start: cue.start,
                        new_end: cue.end,
                        old_end: cue.end,
                    }]);

                    for change in &mut changes {
                        if let Some((i, cue)) =
                            cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                        {
                            change.new_index = i;
                            change.new_start = cue.start;
                            change.new_end = cue.end;
                        };
                    }

                    document_state.unsaved_changes = true;
                    app.set_current_cue_end_time(position)?;

                    let edit = Edit::EditCueTimes { changes };
                    app.push_to_history(edit);
                }
                SubtitleCues::Cue(cues) => {
                    let cue = &cues[cursor.cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cursor.cue_index,
                        old_index: cursor.cue_index,
                        new_start: cue.start,
                        old_start: cue.start,
                        new_end: cue.end,
                        old_end: cue.end,
                    }]);

                    for change in &mut changes {
                        if let Some((i, cue)) =
                            cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                        {
                            change.new_index = i;
                            change.new_start = cue.start;
                            change.new_end = cue.end;
                        };
                    }

                    document_state.unsaved_changes = true;
                    app.set_current_cue_end_time(position)?;

                    let edit = Edit::EditCueTimes { changes };
                    app.push_to_history(edit);
                }
                SubtitleCues::Line(_) => {}
                SubtitleCues::None => {}
            },
            None => {}
        },

        _ => {}
    }

    Ok(())
}
