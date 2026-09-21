use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueContentChange, Edit},
    mode::{AppMode, Cursor, EditCue},
    renderer::Renderer,
};
use subtitles::subtitles::{SubtitleCues, SubtitleDocument};

pub fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    cursor: Cursor,
    selected_cues: Vec<EditCue>,
    _config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let document = match &mut app.state.subtitle_document {
        Some(document) => document,
        None => {
            app.switch_to_normal_mode();
            return Ok(());
        }
    };

    match key.code {
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,

        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => match &document.cues {
            SubtitleCues::Word(cues) => {
                document.save()?;
                app.state.unsaved_changes = false;
                app.state.subtitle_document = match app.state.track {
                    Some(ref track) => match &track.file_path {
                        Some(file_path) => {
                            let mut lyrics_path = file_path.to_path_buf();
                            lyrics_path.set_extension("lrc");
                            let subtitle_document = SubtitleDocument::from_pathbuf(lyrics_path)?;

                            let selected_cues = selected_cues
                                .iter()
                                .map(|c| EditCue {
                                    index: c.index,
                                    original_content: SubtitleCues::Word(Vec::from([cues
                                        [c.index]
                                        .clone()])),
                                })
                                .collect();

                            app.state.app_mode = AppMode::Edit {
                                cursor,
                                selected_cues,
                            };
                            Some(subtitle_document)
                        }
                        None => None,
                    },
                    None => None,
                };
            }
            SubtitleCues::Cue(cues) => {
                document.save()?;
                app.state.unsaved_changes = false;
                app.state.subtitle_document = match app.state.track {
                    Some(ref track) => match &track.file_path {
                        Some(file_path) => {
                            let mut lyrics_path = file_path.to_path_buf();
                            lyrics_path.set_extension("lrc");
                            let subtitle_document = SubtitleDocument::from_pathbuf(lyrics_path)?;

                            let selected_cues = selected_cues
                                .iter()
                                .map(|c| EditCue {
                                    index: c.index,
                                    original_content: SubtitleCues::Cue(Vec::from([
                                        cues[c.index].clone()
                                    ])),
                                })
                                .collect();

                            app.state.app_mode = AppMode::Edit {
                                cursor,
                                selected_cues,
                            };
                            Some(subtitle_document)
                        }
                        None => None,
                    },
                    None => None,
                };
            }
            SubtitleCues::Line(_) => {}
            SubtitleCues::None => {}
        },

        // Undo and redo changes
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => app.undo(),
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => app.redo(),

        KeyCode::Esc => {
            if app.state.unsaved_changes == true {
                match &mut document.cues {
                    SubtitleCues::Word(subtitle_cues) => {
                        for selected_cue in selected_cues {
                            if let SubtitleCues::Word(cues) = selected_cue.original_content {
                                subtitle_cues[selected_cue.index] = cues[0].clone();
                            }
                        }

                        app.state.unsaved_changes = false;
                        app.state.subtitle_document = match app.state.track {
                            Some(ref track) => match &track.file_path {
                                Some(file_path) => {
                                    let mut lyrics_path = file_path.to_path_buf();
                                    lyrics_path.set_extension("lrc");
                                    SubtitleDocument::from_pathbuf(lyrics_path).ok()
                                }
                                None => None,
                            },
                            None => None,
                        };
                    }
                    SubtitleCues::Cue(subtitle_cues) => {
                        for selected_cue in selected_cues {
                            if let SubtitleCues::Cue(cues) = selected_cue.original_content {
                                subtitle_cues[selected_cue.index] = cues[0].clone();
                            }
                        }

                        app.state.unsaved_changes = false;
                        app.state.subtitle_document = match app.state.track {
                            Some(ref track) => match &track.file_path {
                                Some(file_path) => {
                                    let mut lyrics_path = file_path.to_path_buf();
                                    lyrics_path.set_extension("lrc");
                                    SubtitleDocument::from_pathbuf(lyrics_path).ok()
                                }
                                None => None,
                            },
                            None => None,
                        };
                    }
                    SubtitleCues::Line(subtitle_cues) => {
                        for selected_cue in selected_cues {
                            if let SubtitleCues::Line(cues) = selected_cue.original_content {
                                subtitle_cues[selected_cue.index] = cues[0].clone();
                            }
                        }

                        app.state.unsaved_changes = false;
                        app.state.subtitle_document = match app.state.track {
                            Some(ref track) => match &track.file_path {
                                Some(file_path) => {
                                    let mut lyrics_path = file_path.to_path_buf();
                                    lyrics_path.set_extension("lrc");
                                    SubtitleDocument::from_pathbuf(lyrics_path).ok()
                                }
                                None => None,
                            },
                            None => None,
                        };
                    }
                    SubtitleCues::None => {}
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
            } => cursor.move_right(document.cues.cue_len(cursor.cue_index)),
            _ => {}
        },
        KeyCode::Up => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => cursor.move_up(document.cues.cue_len(cursor.cue_index.saturating_sub(1))),
            _ => {}
        },
        KeyCode::Down => match &mut app.state.app_mode {
            AppMode::Edit {
                cursor,
                selected_cues: _,
            } => {
                let line_count = document.cues.len();
                let new_line_index = std::cmp::min(
                    cursor.cue_index.saturating_add(1),
                    document.cues.len().saturating_sub(1),
                );
                let new_line_length = document.cues.cue_len(new_line_index);
                cursor.move_down(new_line_length, line_count)
            }
            _ => {}
        },

        KeyCode::Char(char) => match &mut document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                app.state.unsaved_changes = true;
                current_cue.words.last_mut().map(|l| l.content.push(char));

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

                app.state.unsaved_changes = true;
                current_cue.content.push(char);

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

                app.state.unsaved_changes = true;
                current_cue.content.push(char);

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
        KeyCode::Backspace => match &mut document.cues {
            SubtitleCues::Word(cues) => {
                let current_cue = &mut cues[cursor.cue_index];
                let old_content = SubtitleCues::Word(Vec::from([current_cue.clone()]));

                app.state.unsaved_changes = true;
                current_cue.words.last_mut().map(|w| w.content.pop());

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

                app.state.unsaved_changes = true;
                current_cue.content.pop();

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

                app.state.unsaved_changes = true;
                current_cue.content.pop();

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
