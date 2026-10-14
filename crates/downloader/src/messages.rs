use mpris::track::Track;
use subtitles::subtitles::SubtitleDocument;

use crate::{error::DownloadError, service::DownloadService};

pub enum DownloadRequest {
    Download(DownloadTask),
    // Cancel,
}

#[derive(Debug)]
pub struct DownloadTask {
    pub track: Track,
    pub download_service: DownloadService,
}

// Change error to not be string
#[derive(Debug)]
pub enum DownloadResult {
    Complete(Option<SubtitleDocument>),
    Cancelled,
    Failed(DownloadError),
}
