use std::fmt::Debug;

use lyrc_core::{mode::AppMode, state::AppState};
use ratatui::{Frame, layout::Rect, widgets::Paragraph};

pub fn draw_footer<A: Debug>(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    active_cues: &[A],
) {
    let automatic_scroll_offset = state.automatic_scroll_offset;
    let mode = state.app_mode.to_string();

    let text = match state.app_mode {
        AppMode::Normal => format!(
            "automatic scroll: {:?}, active cues: {:?}\nmode: {:?}",
            automatic_scroll_offset, active_cues, mode,
        ),
        AppMode::Select { cue_index, selected_cues: _ } => format!(
            "selected line: {:?}, automatic scroll: {:?}\nactive cues: {:?} mode: {:?}",
            cue_index, automatic_scroll_offset, active_cues, mode,
        ),
        AppMode::Edit { cursor, selected_cues: _ } => format!(
            "cursor: {:?},\nautomatic scroll: {:?}\nactive cues: {:?} mode: {:?}",
            cursor, automatic_scroll_offset, active_cues, mode,
        ),
    };

    frame.render_widget(Paragraph::new(text), area);
}
