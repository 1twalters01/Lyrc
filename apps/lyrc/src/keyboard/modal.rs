use std::str::FromStr;

use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    modal::{Modal, ModalError},
    renderer::Renderer,
    state::{SubtitleDocumentState, SubtitleVariant},
};
use subtitles::{
    language::Language,
    subtitles::{SubtitleDocument, SyncLevel},
};

pub async fn handle_key<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    modal: Modal,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match modal {
        Modal::Translate {
            mut input,
            mut input_variant,
            mut new_variant,
            mut error,
        } => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.state.quit = true,
            KeyCode::Esc => app.state.modal = None,

            KeyCode::Tab => {
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
            KeyCode::Esc => app.state.modal = None,

            KeyCode::Tab => {
                app.state.modal = Some(Modal::Translate {
                    input: String::new(),
                    input_variant: None,
                    new_variant: None,
                    error: None,
                })
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
