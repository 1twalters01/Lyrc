use chrono::Duration;
use lyrc_core::state::AppState;
use ratatui::{Frame, layout::Rect, widgets::Paragraph};

pub fn draw_header(frame: &mut Frame, area: Rect, state: &AppState, position: Option<Duration>) {
    let track = state.track.as_ref();
    let title = track
        .map(|track| track.title.as_str())
        .unwrap_or_else(|| "");

    let artist = track
        .and_then(|track| track.artists.first())
        .cloned()
        .unwrap_or_else(|| String::from(""));

    let duration = track
        .and_then(|track| {
            format!(
                "{}:{:02}",
                track.duration.num_minutes(),
                track.duration.num_seconds() % 60,
            )
            .into()
        })
        .unwrap_or_else(|| String::from(""));

    let position_str = match position {
        Some(position) => format!(
            "{}:{:02}",
            position.num_minutes(),
            position.num_seconds() % 60,
        ),
        None => String::from("Not Playing"),
    };

    let mut header = format!("{} - {} - {}/{}", title, artist, position_str, duration);
    if let Some(document_state) = state.subtitle_documents.active() {
        if document_state.unsaved_changes {
            header.push_str("\nunsaved changes");
        }
    }

    frame.render_widget(Paragraph::new(header), area);
}
