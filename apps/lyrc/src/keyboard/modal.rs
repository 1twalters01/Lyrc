use std::str::FromStr;

use configuration::config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lyrc_core::{
    app::App,
    modal::{Modal, ModalError},
    renderer::Renderer,
    state::SubtitleVariant,
};
use subtitles::{language::Language, subtitles::SyncLevel};

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
        } => {}
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
