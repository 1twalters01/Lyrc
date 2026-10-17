use std::{cell::RefCell, sync::Arc};
use std::time::Duration;

use configuration::config::Config;
use database::service::DatabaseService;
use gtk::{gio::prelude::ApplicationExt, glib::MainContext};
use gui::{
    application::{build_application, run_application},
    renderer::GuiRenderer,
};
use lyrc_core::{app::App, renderer::Renderer};
use mpris::client::MprisClient;

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

    let database_service_rc_refcell =
        RefCell::new(Some(database_service));

    let download_tx = RefCell::new(Some(workers.tx.download_tx));
    let translation_tx =
        RefCell::new(Some(workers.tx.translation_tx));
    let alignment_tx = RefCell::new(Some(workers.tx.alignment_tx));

    let (tx, rx) = tokio::sync::mpsc::channel::<AppMessage>(32);
    let rx = RefCell::new(Some(rx));
    let tx_mpris = tx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            if let Some(ref client) = *mpris_client_bg {
                let current_track = client.get_current_track().await;
                let current_position = client.get_current_position().await;

                if let (Ok(track), Ok(position)) =
                    (current_track, current_position)
                {
                    let _ = tx_mpris
                        .send(AppMessage::TrackUpdated {
                            track: Some(track),
                            position: Some(position),
                        })
                        .await;
                } else {
                    let _ = tx_mpris
                        .send(AppMessage::TrackUpdated {
                            track: None,
                            position: None,
                        })
                        .await;
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
            let mut rx = rx;
            while let Some(message) = rx.recv().await {
                match message {
                    AppMessage::TrackUpdated { track, position } => {
                        app.state.track = track;

                        let active_indices =
                            app.synchronizer.get_active_indices();
                        if let Err(e) = app.renderer.render(
                            &mut app.state,
                            position,
                            &active_indices,
                        ) {
                            eprintln!("Render error: {e}");
                        }
                    }
                }
            }
        });
    });

    run_application(&application);

    Ok(())
}
