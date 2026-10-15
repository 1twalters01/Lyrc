use chrono::Duration;
use lyrc_core::{
    renderer::Renderer, state::AppState, synchronizer::ActiveIndex,
};
use std::io::{self, Stdout};

use ratatui::{
    Terminal,
    crossterm::{
        execute,
        terminal::{
            EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
            enable_raw_mode,
        },
    },
    prelude::CrosstermBackend,
};

use crate::draw::window::{draw_window, window_layout};

pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    lines_per_page: usize,
}

impl TuiRenderer {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;

        let mut stdout = std::io::stdout();

        execute!(stdout, EnterAlternateScreen,)?;

        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;

        let lines_per_page = 0;

        Ok(Self {
            terminal,
            lines_per_page,
        })
    }
}

impl Renderer for TuiRenderer {
    type Error = std::io::Error;

    fn render(
        &mut self,
        state: &mut AppState,
        position: Option<Duration>,
        active_cues: &Vec<ActiveIndex>,
    ) -> Result<(), Self::Error> {
        self.terminal.draw(|frame| {
            let window_layout = window_layout(frame);
            self.lines_per_page = window_layout[1].height as usize;

            draw_window(frame, window_layout, state, position, active_cues);
        })?;

        Ok(())
    }

    fn get_lines_per_page(&self) -> usize {
        self.lines_per_page
    }
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen,);
    }
}
