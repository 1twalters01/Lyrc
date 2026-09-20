use std::fmt::Debug;

use lyrc_core::{mode::AppMode, state::AppState};
use ratatui::{Frame, layout::Rect, widgets::Paragraph};

pub fn draw_footer<A: Debug>(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    active_cues: &[A],
) {
    let selected_cue = match state.app_mode {
        AppMode::Normal => None,
        AppMode::Select {
            cue_index,
            selected_cues: _,
        } => Some(cue_index),
        AppMode::Edit {
            cursor,
            selected_cues: _,
        } => Some(cursor.cue_index),
    };

    let automatic_scroll_offset = state.automatic_scroll_offset;
    let mode = state.app_mode.to_string();

    let text = format!(
        "\nselected line: {:?}\nautomatic scroll: {:?}\nactive cues: {:?}\nmode: {:?}",
        selected_cue, automatic_scroll_offset, active_cues, mode,
    );

    frame.render_widget(Paragraph::new(text), area);
}
