use lyrc_core::{mode::AppMode, state::AppState, synchronizer::ActiveIndex};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use subtitles::subtitles::SubtitleCues;

pub fn draw_body(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    active_indices: &Vec<ActiveIndex>,
) {
    // println!("active indices {:?}", active_indices);
    let visible_height = area.height as usize;
    let middle = visible_height / 2;

    let subtitle_document_state = state.subtitle_documents.active();
    let mut lines: Vec<Line> = subtitle_document_state
        .map(|document_state| match &document_state.document.cues {
            SubtitleCues::Word(cues) => cues
                .iter()
                .map(|cue| {
                    let start_timestamp = format!(
                        "{:02}:{:02}.{:03}",
                        cue.start.num_minutes(),
                        cue.start.num_seconds() % 60,
                        (cue.start.num_milliseconds() % 1000) / 10,
                    );

                    let end_timestamp = format!(
                        "{:02?}:{:02?}.{:03?}",
                        cue.end.num_minutes(),
                        cue.end.num_seconds() % 60,
                        (cue.end.num_milliseconds() % 1000) / 10,
                    );

                    let mut line = Line::from(Span::raw(format!(
                        "[{} - {}] ",
                        start_timestamp, end_timestamp
                    )));

                    line.extend(cue.words.iter().map(|word| {
                        let mut spaced_word = word.clone();
                        spaced_word.content.push_str(" ");
                        Span::raw(spaced_word.content)
                    }));
                    line
                })
                .collect(),
            SubtitleCues::Cue(cues) => cues
                .iter()
                .map(|cue| {
                    let start_timestamp = format!(
                        "{:02}:{:02}.{:03}",
                        cue.start.num_minutes(),
                        cue.start.num_seconds() % 60,
                        (cue.start.num_milliseconds() % 1000) / 10,
                    );

                    let end_timestamp = format!(
                        "{:02?}:{:02?}.{:03?}",
                        cue.end.num_minutes(),
                        cue.end.num_seconds() % 60,
                        (cue.end.num_milliseconds() % 1000) / 10,
                    );
                    Line::from(format!(
                        "[{} - {}] {}",
                        start_timestamp, end_timestamp, cue.content
                    ))
                })
                .collect(),
            SubtitleCues::Line(cues) => cues
                .iter()
                .map(|cue| Line::from(format!("{}", cue.content)))
                .collect(),
            SubtitleCues::None => Vec::new(),
        })
        .unwrap_or_default();

    let (selected_cue, selected_cues) = match &state.app_mode {
        AppMode::Normal => (None, Vec::new()),
        AppMode::Select {
            cue_index,
            selected_cues,
        } => (Some(*cue_index), selected_cues.clone()),
        AppMode::Edit {
            cursor,
            selected_cues,
        } => (
            Some(cursor.cue_index),
            selected_cues.iter().map(|c| c.index).collect(),
        ),
    };

    let selected_line_indices = match selected_cue {
        None => &Vec::new(),
        Some(index) => &Vec::from([index]),
    };

    highlight_lines(
        &mut lines,
        &active_indices.iter().map(|i| i.cue_index().cue).collect(),
        selected_line_indices,
        &selected_cues,
    );

    highlight_words(&mut lines, active_indices);

    let automatic_scroll_offset = active_indices
        .iter()
        .map(|i| i.cue_index().cue)
        .collect::<Vec<_>>()
        .first()
        .unwrap_or(&0usize)
        .saturating_sub(middle);
    // println!("active indices: {:?}", active_indices);
    // println!("offset: {:?}", automatic_scroll_offset);
    state.automatic_scroll_offset = automatic_scroll_offset;

    let scroll_offset = match selected_cue {
        Some(offset) => {
            if offset > middle {
                offset - middle
            } else {
                0
            }
        }
        None => automatic_scroll_offset,
    };

    frame.render_widget(
        Paragraph::new(lines).scroll((scroll_offset as u16, 0)),
        area,
    );
}

fn highlight_lines(
    lines: &mut [Line],
    bold_indices: &Vec<usize>,
    reverse_indices: &[usize],
    selected_indices: &[usize],
) {
    for &index in bold_indices {
        if let Some(line) = lines.get_mut(index) {
            *line = line
                .clone()
                .patch_style(Style::default().add_modifier(Modifier::BOLD));
        }
    }

    for &index in reverse_indices {
        if let Some(line) = lines.get_mut(index) {
            *line = line
                .clone()
                .patch_style(Style::default().add_modifier(Modifier::REVERSED));
        }
    }

    for &index in selected_indices {
        if let Some(line) = lines.get_mut(index) {
            *line = line
                .clone()
                .patch_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }
    }
}

fn highlight_words(lines: &mut [Line], active_indices: &Vec<ActiveIndex>) {
    for index in active_indices {
        if let ActiveIndex::Word(index) = index {
            let cue_index = index.cue;
            let word_index = index.word;

            if let Some(line) = lines.get_mut(cue_index) {
                if let Some(span) = line.spans.get_mut(word_index + 1) {
                    span.style = Style::default().red();
                }
            }
        }
    }
}
