use aligner::messages::AlignmentRequest;
use configuration::config::PlaybackConfig;
use database::service::DatabaseService;
use downloader::messages::DownloadRequest;
use mpris::client::MprisClient;
use tokio::sync::mpsc::Sender;
use translator::messages::TranslationRequest;

use crate::{
    clock::PlaybackClock, renderer::Renderer, state::AppState,
    synchronizer::AppSynchronizer,
};

pub struct App<R>
where
    R: Renderer,
{
    pub renderer: R,
    pub database_service: DatabaseService,
    pub synchronizer: AppSynchronizer,
    pub clock: PlaybackClock,
    pub state: AppState,
    pub mpris_client: Option<MprisClient>,
    pub download_req_tx: Sender<DownloadRequest>,
    pub translation_req_tx: Sender<TranslationRequest>,
    pub alignment_req_tx: Sender<AlignmentRequest>,
}

impl<R> App<R>
where
    R: Renderer,
{
    pub fn new(
        renderer: R,
        database_service: DatabaseService,
        mpris_client: Option<MprisClient>,
        download_req_tx: Sender<DownloadRequest>,
        translation_req_tx: Sender<TranslationRequest>,
        alignment_req_tx: Sender<AlignmentRequest>,
        playback_config: &PlaybackConfig,
    ) -> Self {
        let synchronizer = AppSynchronizer::new();
        let clock = PlaybackClock::new(playback_config.clock_offset);
        let state = AppState::new();

        Self {
            renderer,
            database_service,
            clock,
            state,
            synchronizer,
            mpris_client,
            download_req_tx,
            translation_req_tx,
            alignment_req_tx,
        }
    }

    pub async fn update(&mut self) {
        self.update_track().await;
        let _ = self.update_subtitle_document().await;
    }
}
