use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    history::{CueTimeChange, Edit},
    renderer::Renderer, state::{SubtitleDocumentState, SubtitleVariant},
};
use lyrics::{models::LyricsFormat, service::LyricsService};
use subtitles::{
    formats::lrc::parser::LrcParser,
    parser::SubtitleParser,
    subtitles::SubtitleCues,
};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        // Quit
        KeyCode::Esc => {
            let document_state = app.state.subtitle_documents.active_mut();
            if let Some(document_state) = document_state {
                if document_state.unsaved_changes {
                    document_state.unsaved_changes = false;
                    app.state.reload_subtitle_documents().await;
                    if let Some(active_variant) = app.state.subtitle_documents.active_variant() {
                        app.state.subtitle_documents.set_language(active_variant);
                    }
                }
            } else {
                app.state.quit = true
            }
        }
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,

        // Save
        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
            match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    document_state.document.save()?;
                    document_state.unsaved_changes = false;
                }
                None => {}
            }
        }

        // Undo and redo changes
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => app.undo(),
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => app.redo(),

        // playback control
        KeyCode::Char(' ') => app.toggle_play_pause().await?,
        KeyCode::Left => app.seek_by_duration(config.rewind_duration).await?,
        KeyCode::Right => app.seek_by_duration(config.fast_forward_duration).await?,

        // line control
        // Use the app.select_next_line and app.select_next_line
        KeyCode::Up => app.go_to_previous_line(),
        KeyCode::Down => app.go_to_next_line(),
        KeyCode::Char('k') => app.go_to_previous_line(),
        KeyCode::Char('j') => app.go_to_next_line(),
        KeyCode::Char('h') => app.toggle_select_all_lines()?,

        // Change modes
        KeyCode::Enter => app.switch_to_select_mode()?,
        KeyCode::Tab => app.switch_to_select_mode()?,

        // Bulk adjust cue times
        KeyCode::Char('m') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                        SubtitleCues::Word(cues) => cues
                            .iter()
                            .enumerate()
                            .map(|(i, cue)| CueTimeChange {
                                id: cue.id.clone(),
                                new_index: i,
                                old_index: i,
                                new_start: cue.start,
                                old_start: cue.start,
                                new_end: cue.end,
                                old_end: cue.end,
                            })
                            .collect::<Vec<CueTimeChange>>(),
                        SubtitleCues::Cue(cues) => cues
                            .iter()
                            .enumerate()
                            .map(|(i, cue)| CueTimeChange {
                                id: cue.id.clone(),
                                new_index: i,
                                old_index: i,
                                new_start: cue.start,
                                old_start: cue.start,
                                new_end: cue.end,
                                old_end: cue.end,
                            })
                            .collect::<Vec<CueTimeChange>>(),
                        SubtitleCues::Line(_) => Vec::new(),
                        SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;

                    changes
                },
                None => Vec::new(),
            };

            app.decrease_all_cue_start_times(config.backwards_cue_increment_small);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char(',') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                        SubtitleCues::Word(cues) => cues
                            .iter()
                            .enumerate()
                            .map(|(i, cue)| CueTimeChange {
                                id: cue.id.clone(),
                                new_index: i,
                                old_index: i,
                                new_start: cue.start,
                                old_start: cue.start,
                                new_end: cue.end,
                                old_end: cue.end,
                            })
                            .collect::<Vec<CueTimeChange>>(),
                        SubtitleCues::Cue(cues) => cues
                            .iter()
                            .enumerate()
                            .map(|(i, cue)| CueTimeChange {
                                id: cue.id.clone(),
                                new_index: i,
                                old_index: i,
                                new_start: cue.start,
                                old_start: cue.start,
                                new_end: cue.end,
                                old_end: cue.end,
                            })
                            .collect::<Vec<CueTimeChange>>(),
                        SubtitleCues::Line(_) => Vec::new(),
                        SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;

                    changes
                },
                None => Vec::new(),
            };

            app.increase_all_cue_start_times(config.forwards_cue_increment_small);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('.') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;

                    changes
                },
                None => Vec::new(),
            };

            app.decrease_all_cue_end_times(config.backwards_cue_increment_small);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('/') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;
                    changes
                },
                None => Vec::new(),
            };

            app.increase_all_cue_end_times(config.forwards_cue_increment_small);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('M') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;
                    changes
                },
                None => Vec::new(),
            };

            app.decrease_all_cue_start_times(config.backwards_cue_increment_large);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('<') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;
                    changes
                },
                None => Vec::new(),
            };

            app.increase_all_cue_start_times(config.forwards_cue_increment_large);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('>') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;
                    changes
                },
                None => Vec::new(),
            };

            app.decrease_all_cue_end_times(config.backwards_cue_increment_large);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut *changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes: changes };
            app.push_to_history(edit);
        }
        KeyCode::Char('?') => {
            let mut changes = match &mut app.state.subtitle_documents.active_mut() {
                Some(document_state) => {
                    let changes = match &document_state.document.cues {
                    SubtitleCues::Word(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Cue(cues) => cues
                        .iter()
                        .enumerate()
                        .map(|(i, cue)| CueTimeChange {
                            id: cue.id.clone(),
                            new_index: i,
                            old_index: i,
                            new_start: cue.start,
                            old_start: cue.start,
                            new_end: cue.end,
                            old_end: cue.end,
                        })
                        .collect::<Vec<CueTimeChange>>(),
                    SubtitleCues::Line(_) => Vec::new(),
                    SubtitleCues::None => Vec::new(),
                    };

                    document_state.unsaved_changes = true;
                    changes
                },
                None => Vec::new(),
            };

            app.increase_all_cue_end_times(config.forwards_cue_increment_large);

            if let Some(document_state) = &mut app.state.subtitle_documents.active_mut() {
                match &document_state.document.cues {
                    SubtitleCues::Word(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Cue(cues) => {
                        for change in &mut changes {
                            if let Some((i, cue)) =
                                cues.iter().enumerate().find(|(_, cue)| cue.id == change.id)
                            {
                                change.new_index = i;
                                change.new_start = cue.start;
                                change.new_end = cue.end;
                            };
                        }
                    }
                    SubtitleCues::Line(_) => {}
                    SubtitleCues::None => {}
                }
            }

            let edit = Edit::EditCueTimes { changes };
            app.push_to_history(edit);
        }

        // align lyrics
        KeyCode::Char('a') => app.start_alignment().await?,

        // download lyrics
        KeyCode::Char('d') => {
            if app.state.subtitle_documents.get_original().is_none() {
                // store in app? and have app.lyrics_service or something?
                let lyrics_service = LyricsService::default();
                let lyrics_provider = lyrics_service.providers.get("lrclib");
                let track = app.state.track.clone();
                let subtitle_document = match (lyrics_provider, track) {
                    (Some(provider), Some(track)) => {
                        let lyrics = provider.search(track.clone()).await?;
                        let mut document_path = None;
                        if let Some(lyrics) = lyrics {
                            match lyrics.format {
                                LyricsFormat::Lrc => {
                                    if let Some(file_path) = track.file_path {
                                        let mut lrc_path = file_path.to_path_buf();
                                        lrc_path.set_extension("lrc");
                                        document_path = Some(lrc_path);
                                    }
                                    let mut document = LrcParser.parse(&lyrics.content)?;
                                    document.metadata.file_path = document_path;

                                    Some(document)
                                }
                                LyricsFormat::Text => None,
                            }
                        } else {
                            None
                        }
                    }
                    (_, _) => None,
                };

                if let Some(subtitle_document) = subtitle_document {
                    app.state.subtitle_documents.insert(
                        SubtitleVariant::Original,
                        SubtitleDocumentState::new(subtitle_document),
                    );
                }

                if app.state.subtitle_documents.active().is_none() {
                    app.state.subtitle_documents.select_default();
                }
            }
        }

        _ => {}
    }

    Ok(())
}
