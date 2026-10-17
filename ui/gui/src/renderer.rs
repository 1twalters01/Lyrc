use std::io;

use chrono::Duration;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation,
    prelude::{BoxExt, GtkWindowExt},
};
use lyrc_core::renderer::Renderer;

pub struct GuiRenderer {
    window: ApplicationWindow,
    position_label: Label,
    artist_label: Label,
    lines_per_page: usize,
}

impl GuiRenderer {
    pub fn new(application: &Application) -> io::Result<Self> {
        let position_label = Label::new(Some("00:00"));
        let artist_label = Label::new(None);

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.append(&position_label);
        content.append(&artist_label);

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Lyrc")
            .child(&content)
            .build();

        Ok(Self {
            window,
            position_label,
            artist_label,
            lines_per_page: 0,
        })
    }

    pub fn present(&self) {
        self.window.present();
    }
}

impl Renderer for GuiRenderer {
    type Error = io::Error;

    fn render(
        &mut self,
        state: &mut lyrc_core::state::AppState,
        position: Option<Duration>,
        active_indices: &Vec<lyrc_core::synchronizer::ActiveIndex>,
    ) -> Result<(), Self::Error> {
        let time_text = position
            .map(|duration| {
                let total_seconds = duration.num_seconds();
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;

                format!("{minutes:02}:{seconds:02}")
            })
            .unwrap_or_else(|| "--:--".to_string());

        self.position_label.set_text(&time_text);

        if let Some(ref track) = state.track {
            self.artist_label.set_text(&track.artists.join(", "));
        }

        Ok(())
    }

    fn get_lines_per_page(&self) -> usize {
        self.lines_per_page
    }
}
