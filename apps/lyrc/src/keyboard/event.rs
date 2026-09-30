use configuration::config::Config;
use crossterm::event::KeyEvent;
use lyrc_core::{app::App, mode::AppMode, renderer::Renderer};

use crate::keyboard;

pub async fn handle_keyboard_event<R: Renderer>(
    app: &mut App<R>,
    key: KeyEvent,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(modal) = &app.state.modal {
        return keyboard::modal::handle_key(app, key, modal.clone(), config).await;
    }

    match &app.state.app_mode {
        AppMode::Normal => keyboard::normal::handle_key(app, key, &config).await,
        AppMode::Select {
            cursor,
            selected_cues: _,
        } => keyboard::select::handle_key(app, key, &config).await,
        AppMode::Edit {
            cursor: _,
            selected_cues: _,
        } => keyboard::edit::handle_key(app, key, &config).await,
    }
}
