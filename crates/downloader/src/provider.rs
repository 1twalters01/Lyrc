use crate::{error::DownloadError, models::Lyrics};
use futures::future::BoxFuture;
use mpris::track::Track;
use pyo3_async_runtimes::TaskLocals;
use subtitles::subtitles::SubtitleDocument;

pub trait LyricsDownloader: Send + Sync {
    fn search(
        &self,
        track: Track,
        locals: TaskLocals,
    ) -> BoxFuture<'_, Result<Option<Lyrics>, DownloadError>>;
    // fn search(&self, track: Track) -> BoxFuture<'_,
    // Result<Option<SubtitleDocument>, DownloadError>>;
}
