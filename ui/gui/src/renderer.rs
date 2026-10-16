use std::io;

use chrono::Duration;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation,
    prelude::{BoxExt, GtkWindowExt},
};
use lyrc_core::renderer::Renderer;

pub struct GuiRenderer {
    window: ApplicationWindow,
    subtitle_label: Label,
    translation_label: Label,
    lines_per_page: usize,
}

impl GuiRenderer {
    pub fn new(application: &Application) -> io::Result<Self> {
        let subtitle_label = Label::new(Some("hi"));
        let translation_label = Label::new(Some("test"));

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.append(&subtitle_label);
        content.append(&translation_label);

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Lyrc")
            .child(&content)
            .build();

        Ok(Self {
            window,
            subtitle_label,
            translation_label,
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
        todo!()
    }

    fn get_lines_per_page(&self) -> usize {
        self.lines_per_page
    }
}
