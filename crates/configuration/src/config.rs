use chrono::Duration;

pub struct Config {
    pub targets_in_priority_order: Vec<String>,
    pub fps: f64,
    pub clock_offset: Duration,
    pub rewind_duration: Duration,
    pub fast_forward_duration: Duration,
    pub forwards_cue_increment_large: Duration,
    pub backwards_cue_increment_large: Duration,
    pub forwards_cue_increment_small: Duration,
    pub backwards_cue_increment_small: Duration,
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        let targets_in_priority_order = Vec::from([String::from("mpv"), String::from("cmus")]);
        let fps = 60f64;
        let clock_offset = Duration::milliseconds(0);
        let rewind_duration = Duration::milliseconds(-5000);
        let fast_forward_duration = Duration::milliseconds(5000);
        let forwards_cue_increment_small = Duration::milliseconds(10);
        let backwards_cue_increment_small = Duration::milliseconds(10);
        let forwards_cue_increment_large = Duration::milliseconds(500);
        let backwards_cue_increment_large = Duration::milliseconds(500);

        let database_url = String::new();
        let max_connections = 5;

        Self {
            targets_in_priority_order,
            fps,
            clock_offset,
            rewind_duration,
            fast_forward_duration,
            forwards_cue_increment_small,
            backwards_cue_increment_small,
            forwards_cue_increment_large,
            backwards_cue_increment_large,
            database_url,
            max_connections,
        }
    }
}
