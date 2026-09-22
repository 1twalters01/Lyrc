use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueTimeChange, Edit, IndexedSubtitleCue},
    mode::AppMode,
    renderer::Renderer,
};
use subtitles::subtitles::{SubtitleCues, SubtitleDocument};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    cue_index: usize,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut document_state = app.state.subtitle_documents.active_mut();
    let document_state = match &mut document_state {
        Some(document_state) => document_state,
        None => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

    match key.code {
        // Quit
        KeyCode::Char('q') => app.state.quit = true,
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
            app.state.quit = true;
        }

        // Save
        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
            document_state.document.save()?;
            document_state.unsaved_changes = false;
            app.state.reload_subtitle_documents().await;
            if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                app.state.subtitle_documents.set_language(active_variant);
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
                    app.state.subtitle_documents.set_language(active_variant);
                }
            } else {
                app.switch_to_normal_mode()
            }
        }
        KeyCode::Tab => app.switch_to_edit_mode()?,
        KeyCode::Enter => app.seek_to_selected_line(cue_index).await?,

        // Playback control
        KeyCode::Char(' ') => app.toggle_play_pause().await?,
        KeyCode::Left => app.seek_by_duration(config.rewind_duration).await?,
        KeyCode::Right => app.seek_by_duration(config.fast_forward_duration).await?,

        // Line control
        KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => app.go_to_previous_half_page(),
        KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => app.go_to_next_half_page(),
        KeyCode::Up => app.go_to_previous_line(),
        KeyCode::Down => app.go_to_next_line(),
        KeyCode::Char('H') => app.toggle_select_all_lines()?,
        KeyCode::Char('h') => app.toggle_select_line(),

        KeyCode::Char('D') => match &mut document_state.document.cues {
            SubtitleCues::Word(cues) => {
                let cues = match &app.state.app_mode {
                    AppMode::Normal => Vec::new(),
                    AppMode::Select {
                        cue_index: _,
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
                        cue_index: _,
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
                        cue_index: _,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                let cue = &cues[cue_index];
                let mut changes = Vec::from([CueTimeChange {
                    id: cue.id.clone(),
                    new_index: cue_index,
                    old_index: cue_index,
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
                    let cue = &cues[cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cue_index,
                        old_index: cue_index,
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
                    let cue = &cues[cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cue_index,
                        old_index: cue_index,
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
                    let cue = &cues[cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cue_index,
                        old_index: cue_index,
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
                    let cue = &cues[cue_index];
                    let mut changes = Vec::from([CueTimeChange {
                        id: cue.id.clone(),
                        new_index: cue_index,
                        old_index: cue_index,
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
