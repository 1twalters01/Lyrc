use std::time::Duration as std_duration;

use crate::{
    args::{Args, Command, Frontend},
    keyboard,
    workers::start::Workers,
};

use configuration::config::Config;
use gui::renderer::GuiRenderer;
use lyrc_core::app::App;
use mpris::client::MprisClient;
use synchronizer::strategies::{cues::CueSynchronizer, words::WordSynchronizer};

use crossterm::event::{Event, EventStream};
use futures_util::stream::StreamExt;

pub async fn run_gui(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut workers = Workers::start().await?;
    let player = &MprisClient::choose_player(&config.targets_in_priority_order).await?;

    let cue_synchronizer = CueSynchronizer::new();
    let word_synchronizer = WordSynchronizer::new();

    let mut app = App::new(
        GuiRenderer::new()?,
        player,
        workers.alignment.request_tx,
        workers.translation.request_tx,
        &config,
    )
    .await;

    app.update_track().await;
    app.update_subtitle_document().await;

    let mpris = app.mpris_client.clone();
    let mut events = mpris.events().await?;

    todo!()
}
