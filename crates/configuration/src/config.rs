use chrono::Duration;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let url = String::from("lyrc.db");
        let max_connections = 5;

        Self {
            url,
            max_connections,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PlayerConfig {
    pub targets_in_priority_order: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        let targets_in_priority_order =
            Vec::from([String::from("mpv"), String::from("cmus")]);

        Self {
            targets_in_priority_order,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PlaybackConfig {
    pub fps: f64,
    pub application_id: String,
    pub clock_offset: Duration,
    pub rewind_duration: Duration,
    pub fast_forward_duration: Duration,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        let fps = 60f64;
        let application_id = String::from("com.lyrc.App");
        let clock_offset = Duration::milliseconds(0);
        let rewind_duration = Duration::milliseconds(-5000);
        let fast_forward_duration = Duration::milliseconds(5000);

        Self {
            fps,
            application_id,
            clock_offset,
            rewind_duration,
            fast_forward_duration,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SubtitleConfig {
    pub forwards_cue_increment_large: Duration,
    pub backwards_cue_increment_large: Duration,
    pub forwards_cue_increment_small: Duration,
    pub backwards_cue_increment_small: Duration,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        let forwards_cue_increment_small = Duration::milliseconds(10);
        let backwards_cue_increment_small = Duration::milliseconds(10);
        let forwards_cue_increment_large = Duration::milliseconds(500);
        let backwards_cue_increment_large = Duration::milliseconds(500);

        Self {
            forwards_cue_increment_large,
            backwards_cue_increment_large,
            forwards_cue_increment_small,
            backwards_cue_increment_small,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub database: DatabaseConfig,
    pub player: PlayerConfig,
    pub playback: PlaybackConfig,
    pub subtitles: SubtitleConfig,
}

impl Default for Config {
    fn default() -> Self {
        let database = DatabaseConfig::default();
        let player = PlayerConfig::default();
        let playback = PlaybackConfig::default();
        let subtitles = SubtitleConfig::default();

        Self {
            database,
            player,
            playback,
            subtitles,
        }
    }
}
