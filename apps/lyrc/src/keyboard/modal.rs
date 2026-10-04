use std::str::FromStr;

use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use downloader::{models::LyricsFormat, service::DownloadService};
use lyrc_core::{
    app::App,
    modal::{Modal, ModalError, ModalOption},
    renderer::Renderer,
    state::{SubtitleDocumentState, SubtitleVariant},
};
use mpris::client::MprisClient;
use subtitles::{
    formats::lrc::parser::LrcParser, language::Language, parser::SubtitleParser,
    subtitles::SyncLevel,
};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    modal: Modal,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match modal {
        Modal::Selection { options, new_modal } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => app.state.modal = None,

            KeyCode::Up => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = match new_modal {
                    ModalOption::Player => Some(Modal::Selection {
                        new_modal: if options.contains(&ModalOption::Alignment) {
                            ModalOption::Alignment
                        } else {
                            ModalOption::Download
                        },
                        options,
                    }),
                    ModalOption::Download => Some(Modal::Selection {
                        options,
                        new_modal: ModalOption::Player,
                    }),
                    ModalOption::Translate => Some(Modal::Selection {
                        new_modal: if options.contains(&ModalOption::Download) {
                            ModalOption::Download
                        } else {
                            ModalOption::Player
                        },
                        options,
                    }),
                    ModalOption::Alignment => Some(Modal::Selection {
                        new_modal: if options.contains(&ModalOption::Translate) {
                            ModalOption::Translate
                        } else {
                            ModalOption::Player
                        },
                        options,
                    }),
                }
            }
            KeyCode::Down => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = match new_modal {
                    ModalOption::Player => Some(Modal::Selection {
                        new_modal: ModalOption::Download,
                        options,
                    }),
                    ModalOption::Download => Some(Modal::Selection {
                        new_modal: if options.contains(&ModalOption::Translate) {
                            ModalOption::Translate
                        } else {
                            ModalOption::Player
                        },
                        options,
                    }),
                    ModalOption::Translate => Some(Modal::Selection {
                        new_modal: if options.contains(&ModalOption::Alignment) {
                            ModalOption::Alignment
                        } else {
                            ModalOption::Player
                        },
                        options,
                    }),
                    ModalOption::Alignment => Some(Modal::Selection {
                        options,
                        new_modal: ModalOption::Player,
                    }),
                }
            }

            KeyCode::Enter => match new_modal {
                ModalOption::Player => {
                    app.state.modal = Some(Modal::Player {
                        players: MprisClient::find_players().await?,
                        new_player: None,
                        error: None,
                    })
                }
                ModalOption::Download => {
                    app.state.modal = Some(Modal::Download {
                        providers: DownloadService::get_providers(config),
                        new_provider: DownloadService::get_prefered_provider(config),
                        error: None,
                    })
                }
                ModalOption::Translate => {
                    app.state.modal = Some(Modal::Translate {
                        input: String::new(),
                        input_variant: None,
                        new_variant: None,
                        error: None,
                    })
                }
                ModalOption::Alignment => {
                    app.state.modal = Some(Modal::Alignment {
                        new_alignment: app
                            .state
                            .subtitle_documents
                            .active()
                            .map(|state| state.document.sync_level())
                            .unwrap_or(SyncLevel::None),
                        error: None,
                    })
                }
            },

            _ => {}
        },
        Modal::Player {
            players,
            new_player,
            error,
        } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = Some(Modal::Selection {
                    options,
                    new_modal: ModalOption::Player,
                });
            }

            KeyCode::Char(' ') => {
                app.state.modal = Some(Modal::Player {
                    players: MprisClient::find_players().await?,
                    new_player: None,
                    error: None,
                })
            }

            KeyCode::Up => {
                let next_player =
                    if let Some(idx) = players.iter().position(|p| Some(p.clone()) == new_player) {
                        match players.get(idx + 1) {
                            Some(player) => Some(player),
                            None => players.first(),
                        }
                    } else {
                        players.last()
                    };
                app.state.modal = Some(Modal::Player {
                    players: players.clone(),
                    new_player: next_player.cloned(),
                    error: None,
                })
            }
            KeyCode::Down => {
                let next_player =
                    if let Some(idx) = players.iter().position(|p| Some(p.clone()) == new_player) {
                        match players.get(idx - 1) {
                            Some(player) => Some(player),
                            None => players.last(),
                        }
                    } else {
                        players.first()
                    };
                app.state.modal = Some(Modal::Player {
                    players: players.clone(),
                    new_player: next_player.cloned(),
                    error: None,
                })
            }

            KeyCode::Enter => {
                if let Some(player) = new_player {
                    app.mpris_client = MprisClient::connect(&player).await?;
                    app.update_track().await;
                    app.state.subtitle_documents.clear_cache();
                    app.update_subtitle_document().await;
                }
            }
            _ => {}
        },
        Modal::Download {
            providers,
            new_provider,
            error,
        } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = Some(Modal::Selection {
                    new_modal: if options.contains(&ModalOption::Translate) {
                        ModalOption::Translate
                    } else {
                        ModalOption::Player
                    },
                    options,
                });
            }

            KeyCode::Up => {
                app.state.modal = Some(Modal::Download {
                    providers,
                    new_provider: match new_provider {
                        DownloadService::LibLrc => DownloadService::Musixmatch,
                        DownloadService::Musixmatch => DownloadService::LibLrc,
                    },
                    error: None,
                });
            }
            KeyCode::Down => {
                app.state.modal = Some(Modal::Download {
                    providers,
                    new_provider: match new_provider {
                        DownloadService::LibLrc => DownloadService::Musixmatch,
                        DownloadService::Musixmatch => DownloadService::LibLrc,
                    },
                    error: None,
                });
            }

            KeyCode::Enter => {
                if app.state.subtitle_documents.get_original().is_none() {
                    let lyrics_provider = new_provider.get_provider();
                    let track = app.state.track.clone();
                    let subtitle_document = match track {
                        Some(track) => {
                            let lyrics = lyrics_provider.search(track.clone()).await?;
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
                        _ => None,
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
        },
        Modal::Translate {
            mut input,
            mut input_variant,
            mut new_variant,
            mut error,
        } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = Some(Modal::Selection {
                    new_modal: if options.contains(&ModalOption::Translate) {
                        ModalOption::Translate
                    } else {
                        ModalOption::Player
                    },
                    options,
                });
            }

            KeyCode::Char(char) => {
                input.push(char);
                input_variant = parse_variant(&input);

                app.state.modal = Some(Modal::Translate {
                    input,
                    input_variant,
                    new_variant,
                    error,
                });
            }

            KeyCode::Backspace => {
                input.pop();
                input_variant = parse_variant(&input);

                app.state.modal = Some(Modal::Translate {
                    input,
                    input_variant,
                    new_variant,
                    error,
                });
            }

            KeyCode::Enter => {
                if let Some(input_variant) = input_variant {
                    app.state.modal = Some(Modal::Translate {
                        input: String::new(),
                        input_variant: None,
                        new_variant: Some(input_variant),
                        error,
                    });
                } else if let Some(new_variant) = new_variant {
                    match new_variant {
                        SubtitleVariant::Original => {
                            let res = app
                                .state
                                .subtitle_documents
                                .set_variant(SubtitleVariant::Original);
                            if !res {
                                app.state.modal = Some(Modal::Translate {
                                    input,
                                    input_variant,
                                    new_variant: Some(new_variant),
                                    error: Some(ModalError::InvalidVariant),
                                });
                            } else {
                                app.state.modal = None;
                            }
                        }
                        SubtitleVariant::Translated(language) => {
                            let variant = SubtitleVariant::Translated(language.clone());

                            if app.state.subtitle_documents.set_variant(variant) {
                                app.state.modal = None;
                            } else {
                                app.start_translation(language).await?;
                            }
                        }
                    }
                }
            }

            _ => {}
        },
        Modal::Alignment {
            new_alignment,
            error,
        } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => {
                let options = match app.state.subtitle_documents.active() {
                    Some(_) => Vec::from([
                        ModalOption::Player,
                        ModalOption::Download,
                        ModalOption::Alignment,
                        ModalOption::Translate,
                    ]),
                    None => Vec::from([ModalOption::Player, ModalOption::Download]),
                };
                app.state.modal = Some(Modal::Selection {
                    new_modal: if options.contains(&ModalOption::Translate) {
                        ModalOption::Alignment
                    } else {
                        ModalOption::Player
                    },
                    options,
                });
            }

            KeyCode::Up => {
                app.state.modal = Some(Modal::Alignment {
                    new_alignment: new_alignment.next_cyclic(),
                    error: None,
                })
            }
            KeyCode::Down => {
                app.state.modal = Some(Modal::Alignment {
                    new_alignment: new_alignment.previous_cyclic(),
                    error: None,
                })
            }

            KeyCode::Enter => {
                let mut current_alignment = app
                    .state
                    .subtitle_documents
                    .active()
                    .map(|state| state.document.sync_level())
                    .unwrap_or(SyncLevel::None);

                let Some(variant) = app.state.subtitle_documents.active_variant() else {
                    return Ok(());
                };

                if let Some(document_state) = app.state.subtitle_documents.active() {
                    app.state
                        .subtitle_documents
                        .insert_cache(variant.clone(), document_state.clone());
                }

                if new_alignment <= current_alignment {
                    if let Some(state) = app.state.subtitle_documents.active_mut() {
                        state.document.downgrade(new_alignment)?;
                    }

                    app.state.modal = None;
                    return Ok(());
                }

                if let Some(cached_document_state) =
                    app.state.subtitle_documents.get_from_cache(&variant)
                {
                    let cached_alignment = cached_document_state.document.sync_level();

                    if cached_alignment > current_alignment {
                        app.state
                            .subtitle_documents
                            .insert(variant.clone(), cached_document_state.clone());
                        current_alignment = cached_alignment
                    }

                    if new_alignment <= current_alignment {
                        if new_alignment < current_alignment {
                            if let Some(state) = app.state.subtitle_documents.active_mut() {
                                state.document.downgrade(new_alignment)?;
                            }
                        }

                        app.state.modal = None;
                        return Ok(());
                    }
                }

                app.start_alignment(new_alignment).await?
            }

            _ => {}
        },
    }

    Ok(())
}

fn parse_variant(input: &str) -> Option<SubtitleVariant> {
    if input.to_lowercase() == "original" {
        Some(SubtitleVariant::Original)
    } else {
        Language::from_str(&input)
            .ok()
            .map(|language| SubtitleVariant::Translated(language))
    }
}
