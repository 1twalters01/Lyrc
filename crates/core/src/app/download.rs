
use downloader::{messages::{DownloadRequest, DownloadTask}, service::DownloadService};
use mpris::track::Track;

use crate::{app::App, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn start_download(
        &mut self,
        track: &Track,
        download_service: DownloadService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.state.download_running {
            return Ok(());
        }

        let task = DownloadTask {
            track: track.clone(),
            download_service,
        };

        self.download_req_tx
            .send(DownloadRequest::Download(task))
            .await?;

        self.state.alignment_running = true;

        Ok(())
    }
}
