use crate::{interfaces::tui::events::handle_tui_events, workers::start::Workers};

use configuration::config::Config;
use lyrc_core::app::App;
use mpris::client::MprisClient;
use tui::renderer::TuiRenderer;

pub async fn run_tui(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let workers = Workers::start().await?;

    let player = &MprisClient::choose_player(&config.targets_in_priority_order).await?;

    let app = App::new(
        TuiRenderer::new()?,
        player,
        workers.alignment.request_tx.clone(),
        workers.translation.request_tx.clone(),
        &config,
    )
    .await;

    handle_tui_events(app, workers, &config).await
}
