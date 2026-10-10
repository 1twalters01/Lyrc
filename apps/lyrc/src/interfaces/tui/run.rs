use crate::{interfaces::tui::events::handle_tui_events, workers::start::Workers};

use configuration::config::Config;
use database::service::DatabaseService;
use lyrc_core::app::App;
use mpris::client::MprisClient;
use tui::renderer::TuiRenderer;

pub async fn run_tui(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let player = MprisClient::choose_player(&config.targets_in_priority_order).await?;
    let database_service = DatabaseService::new(&config).await?;
    let workers = Workers::start().await?;

    let app = App::new(
        TuiRenderer::new()?,
        database_service,
        &player,
        workers.tx.alignment_tx,
        workers.tx.translation_tx,
        &config,
    )
    .await;

    handle_tui_events(app, workers.rx, &config).await
}
