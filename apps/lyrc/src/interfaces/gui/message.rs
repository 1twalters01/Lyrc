use chrono::Duration;
use mpris::{playback::PlaybackStatus, track::Track};

#[derive(Debug)]
pub enum AppMessage {
    TickEvent {
        new_position: Option<Duration>,
        playback_status: PlaybackStatus,
    },
}
