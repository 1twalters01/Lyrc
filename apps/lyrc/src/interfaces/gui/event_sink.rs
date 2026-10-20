use std::{cell::RefCell, rc::Rc};

use async_channel::Receiver;
use gtk::glib::MainContext;
use gui::renderer::GuiRenderer;
use lyrc_core::app::App;
use mpris::playback::PlayerEvent;

use crate::interfaces::gui::message::AppMessage;

pub fn event_sink(
    app: Rc<RefCell<App<GuiRenderer>>>,
    rx: Receiver<AppMessage>,
) {
    MainContext::default().spawn_local(async move {
        while let Ok(message) = rx.recv().await {
            let mut app = app.borrow_mut();

            let result = match message {
                AppMessage::FirstEvent { track } => {
                    app.state.track = track;
                    let _ = app.update_subtitle_document();
                    app.state.reload_subtitle_documents();
                    Ok(())
                }
                AppMessage::PlayerEvent { event } => {
                    let _ = app.state.update(&event);
                    let _ = app.handle_player_event(event.clone());
                    if let PlayerEvent::TrackChanged(_) = event {
                        let _ = app.update_subtitle_document();
                        app.state.reload_subtitle_documents();
                    }
                    Ok(())
                }
                AppMessage::TickEvent {
                    new_position,
                    playback_status,
                } => app.process_tick(new_position, playback_status),
                AppMessage::DownloadEvent { result } => {
                    app.handle_download_event(result)
                }
                AppMessage::AlignmentEvent { result } => {
                    app.handle_alignment_event(result)
                }
                AppMessage::TranslationEvent { result } => {
                    app.handle_translation_event(result)
                }
            };

            if let Err(e) = result {
                eprintln!("Event handling error: {e}");
            }
        }
    });
}
