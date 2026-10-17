use chrono::Duration;
use mpris::track::Track;

#[derive(Debug)]
pub enum AppMessage {
    TrackUpdated {
        track: Option<Track>,
        position: Option<Duration>,
    },
}
