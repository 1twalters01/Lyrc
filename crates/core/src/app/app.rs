use aligner::messages::AlignmentRequest;
use configuration::config::Config;
use mpris::client::MprisClient;
use tokio::sync::mpsc::Sender;
use translator::messages::TranslationRequest;

use crate::{
    clock::PlaybackClock, renderer::Renderer, state::AppState, synchronizer::AppSynchronizer,
};

pub struct App<R>
where
    R: Renderer,
{
    pub renderer: R,
    pub synchronizer: AppSynchronizer,
    pub clock: PlaybackClock,
    pub state: AppState,
    pub mpris_client: MprisClient,
    pub alignment_req_tx: Sender<AlignmentRequest>,
    pub translation_req_tx: Sender<TranslationRequest>,
}

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn new(
        renderer: R,
        player: &str,
        alignment_req_tx: Sender<AlignmentRequest>,
        translation_req_tx: Sender<TranslationRequest>,
        config: &Config,
    ) -> Self {
        let synchronizer = AppSynchronizer::new();
        let clock = PlaybackClock::new(config.clock_offset);
        let state = AppState::new();
        let mpris_client = MprisClient::connect(player).await.unwrap();

        let mut app = Self {
            renderer,
            clock,
            state,
            synchronizer,
            mpris_client,
            alignment_req_tx,
            translation_req_tx,
        };

        app.update_track().await;
        app.update_subtitle_document().await;

        app
    }
}
