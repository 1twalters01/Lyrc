use std::time::Duration;

use crate::{keyboard, workers::start::WorkersRx};

use configuration::config::Config;
use lyrc_core::app::App;
use tokio::time::interval;
use tui::renderer::TuiRenderer;

use crossterm::event::{Event, EventStream};
use futures_util::stream::StreamExt;

pub async fn handle_tui_events(
    mut app: App<TuiRenderer>,
    mut workers_rx: WorkersRx,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mpris_client = app.mpris_client.clone();
    let mut mpris_event = match mpris_client.as_ref() {
        Some(mpris_client) => Some(mpris_client.events().await?),
        None => None,
    };

    let mut tick_event = interval(Duration::from_secs_f64(1f64 / config.fps));
    let mut keyboard_event = EventStream::new();

    while !app.state.quit {
        tokio::select! {
            Some(event) = async {
                match mpris_event.as_mut() {
                    Some(events) => events.next().await,
                    None => std::future::pending().await,
                }
            } => app.handle_player_event(event).await?,

            _ = tick_event.tick() => app.handle_tick_event().await?,

            Some(Ok(Event::Key(key))) = keyboard_event.next() => keyboard::handle_keyboard_event(&mut app, key, &config).await?,

            Some(result) = workers_rx.download_rx.recv() => app.handle_download_event(result)?,

            Some(result) = workers_rx.alignment_rx.recv() => app.handle_alignment_event(result)?,

            Some(result) = workers_rx.translation_rx.recv() => app.handle_translation_event(result)?,
        }
    }

    Ok(())
}
