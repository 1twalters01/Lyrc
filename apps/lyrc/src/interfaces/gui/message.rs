use chrono::Duration;
use mpris::{
    playback::{PlaybackStatus, PlayerEvent},
    track::Track,
};

#[derive(Debug)]
pub enum AppMessage {
    FirstEvent {
        track: Option<Track>,
    },
    TickEvent {
        new_position: Option<Duration>,
        playback_status: PlaybackStatus,
    },
    PlayerEvent {
        event: PlayerEvent,
    },
}
