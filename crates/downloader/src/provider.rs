use crate::{error::LyricsError, models::Lyrics};
use futures::future::BoxFuture;
use mpris::track::Track;

pub trait LyricsDownloader: Send + Sync {
    fn search(&self, track: Track) -> BoxFuture<'_, Result<Option<Lyrics>, LyricsError>>;
}
