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
    let modal = state.modal.clone();

    let mut text = match state.app_mode {
        AppMode::Normal => format!(
            "automatic scroll: {:?}, active cues: {:?}\nmode: {:?}\nmodal: {:?}",
            automatic_scroll_offset, active_cues, mode, modal,
        ),
        AppMode::Select {
            cursor,
            selected_cues: _,
        } => format!(
            "selected line: {:?}, automatic scroll: {:?}\nactive cues: {:?} mode: {:?}\nmodal: {:?}",
            cursor, automatic_scroll_offset, active_cues, mode, modal,
        ),
        AppMode::Edit {
            cursor,
            selected_cues: _,
        } => format!(
            "cursor: {:?},\nautomatic scroll: {:?}\nactive cues: {:?} mode: {:?}\nmodal: {:?}",
            cursor, automatic_scroll_offset, active_cues, mode, modal,
        ),
    };

    let words = match state.subtitle_documents.active() {
        Some(state) => match &state.document.cues {
            subtitles::subtitles::SubtitleCues::Word(words) => Some(words[0].clone()),
            _ => None,
        },
        None => None,
    };
    text.push_str(&format!(
        "\nword length: {:?}, words: {:?}",
        words.clone().map(|words| words.words.len()),
        words.map(|words| words
            .words
            .iter()
            .map(|word| word.content.clone())
            .collect::<Vec<_>>()),
    ));

    frame.render_widget(Paragraph::new(text), area);
}
