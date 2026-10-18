use std::time::Duration;
use std::{cell::RefCell, sync::Arc};

use configuration::config::Config;
use database::service::DatabaseService;
use gtk::{gio::prelude::ApplicationExt, glib::MainContext};
use gui::{
    application::{build_application, run_application},
    renderer::GuiRenderer,
};
use lyrc_core::{app::App, renderer::Renderer};
use mpris::client::MprisClient;
use tokio::time::interval;

use crate::{interfaces::gui::message::AppMessage, workers::start::Workers};

pub async fn run_gui(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mpris_client =
        match MprisClient::choose_player(&config.targets_in_priority_order)
            .await?
        {
            Some(player) => Some(MprisClient::connect(&player).await?),
            None => None,
        };
    let database_service = DatabaseService::new(&config).await?;
    let workers = Workers::start().await?;

    run_app(database_service, mpris_client, workers, config).await
}

pub async fn run_app(
    database_service: DatabaseService,
    mpris_client: Option<MprisClient>,
    workers: Workers,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mpris_client = Arc::new(mpris_client);
    let mpris_client_bg = Arc::clone(&mpris_client);

    let database_service_rc_refcell = RefCell::new(Some(database_service));

    let download_tx = RefCell::new(Some(workers.tx.download_tx));
    let translation_tx = RefCell::new(Some(workers.tx.translation_tx));
    let alignment_tx = RefCell::new(Some(workers.tx.alignment_tx));

    let (tx, rx) = async_channel::unbounded::<AppMessage>();
    let rx = RefCell::new(Some(rx));
    let tx_mpris = tx.clone();

    tokio::spawn(async move {
        let mut tick_event =
            interval(Duration::from_secs_f64(1f64 / config.fps));

        loop {
            tokio::select! {
                _ = tick_event.tick() => {
                    if let Some(ref client) = *mpris_client_bg {
                        let current_position = client.get_current_position().await;
                        let playback_status = client.get_playback_status().await;

                        if let (Ok(current_position), Ok(playback_status)) =
                            (current_position, playback_status)
                        {
                            let _ = tx_mpris.send(AppMessage::TickEvent {
                                new_position: Some(current_position),
                                playback_status: playback_status,
                            })
                                .await;
                        }
                    }
                }
            }
        }
    });

    let application = build_application(&config.application_id);
    let config_clone = config.clone();

    application.connect_activate(move |application| {
        let renderer = match GuiRenderer::new(application) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Gui renderer error: {error}");
                return;
            }
        };

        let database_service = database_service_rc_refcell
            .borrow_mut()
            .take()
            .expect("database_service initialized twice");
        let download_tx = download_tx
            .borrow_mut()
            .take()
            .expect("download_tx initialized twice");
        let translation_tx = translation_tx
            .borrow_mut()
            .take()
            .expect("translation_tx initialized twice");
        let alignment_tx = alignment_tx
            .borrow_mut()
            .take()
            .expect("alignment_tx initialized twice");
        let rx = rx.borrow_mut().take().expect("rx taken twice");

        let mpris_for_app = (*mpris_client).clone();

        let mut app = App::new(
            renderer,
            database_service,
            mpris_for_app,
            download_tx,
            translation_tx,
            alignment_tx,
            &config_clone,
        );

        app.renderer.present();

        MainContext::default().spawn_local(async move {
            let rx = rx;
            while let Ok(message) = rx.recv().await {
                println!("pre match");
                match message {
                    AppMessage::TickEvent {
                        new_position,
                        playback_status,
                    } => {
                        println!("1. Syncing clock...");
                        if let Err(e) =
                            app.clock.sync(new_position, playback_status)
                        {
                            eprintln!("Clock sync error: {e}");
                        }

                        println!("2. Received tick: {:?}", new_position);
                        app.state.playback_state = playback_status;

                        println!("3. Getting position...");
                        let position = app.clock.get_position();

                        println!("4. Subtitle document");
                        let subtitle_document = &app.state.subtitle_documents.active();
                        app.synchronizer.update(subtitle_document, &position);

                        println!("3. Getting position...");
                        let position = app.clock.get_position();

                        println!("5. Getting active indices...");
                        let active_indices =
                            app.synchronizer.get_active_indices();

                        println!("6. Rendering...");
                        if let Err(e) = app.renderer.render(
                            &mut app.state,
                            position,
                            &active_indices,
                        ) {
                            eprintln!("Render error: {e}");
                        }

                        println!("--- Tick handled successfully ---");
                    }
                }
            }
        });
    });

    run_application(&application);

    Ok(())
}
