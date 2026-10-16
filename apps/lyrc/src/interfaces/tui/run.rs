use crate::{
    interfaces::tui::events::handle_tui_events, workers::start::Workers,
};

use configuration::config::Config;
use database::service::DatabaseService;
use lyrc_core::app::App;
use mpris::client::MprisClient;
use tui::renderer::TuiRenderer;

pub async fn run_tui(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mpris_client =
        match MprisClient::choose_player(&config.targets_in_priority_order)
            .await?
        {
            Some(player) => Some(MprisClient::connect(&player).await?),
            None => None,
        };
    let database_service = DatabaseService::new(&config).await?;
    let workers = Workers::start().await?;

    run_app(database_service, mpris_client, workers, config).await
}

pub async fn run_app(
    database_service: DatabaseService,
    mpris_client: Option<MprisClient>,
    workers: Workers,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(
        TuiRenderer::new()?,
        database_service,
        mpris_client,
        workers.tx.download_tx,
        workers.tx.translation_tx,
        workers.tx.alignment_tx,
        &config,
    );

    app.update().await;
    handle_tui_events(app, workers.rx, &config).await
}
