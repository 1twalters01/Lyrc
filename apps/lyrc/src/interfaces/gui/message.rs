use aligner::messages::AlignmentResult;
use chrono::Duration;
use downloader::messages::DownloadResult;
use mpris::{
    playback::{PlaybackStatus, PlayerEvent},
    track::Track,
};
use translator::messages::TranslationResult;

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
    DownloadEvent {
        result: DownloadResult,
    },
    AlignmentEvent {
        result: AlignmentResult,
    },
    TranslationEvent {
        result: TranslationResult,
    },
}
