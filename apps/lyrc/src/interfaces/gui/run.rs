use std::{cell::RefCell, rc::Rc, sync::Arc};

use configuration::config::Config;
use database::service::DatabaseService;
use gtk::gio::prelude::ApplicationExt;
use gui::{
    application::{build_application, run_application},
    renderer::GuiRenderer,
};
use lyrc_core::app::App;
use mpris::client::MprisClient;

use crate::{
    interfaces::gui::{
        event_sink::event_sink, event_source::event_source, helpers::take_once,
        message::AppMessage,
    },
    workers::start::Workers,
};

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

    let database_service = RefCell::new(Some(database_service));

    let download_tx = RefCell::new(Some(workers.tx.download_tx));
    let translation_tx = RefCell::new(Some(workers.tx.translation_tx));
    let alignment_tx = RefCell::new(Some(workers.tx.alignment_tx));

    let (tx, rx) = async_channel::unbounded::<AppMessage>();
    let rx = RefCell::new(Some(rx));
    let tx_mpris = tx.clone();

    event_source(config.clone(), mpris_client_bg, workers.rx, tx_mpris);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let app_id = config.application_id.clone();
        let application = build_application(&app_id);

        application.connect_activate(move |application| {
            let renderer = match GuiRenderer::new(application) {
                Ok(renderer) => renderer,
                Err(error) => {
                    eprintln!("Gui renderer error: {error}");
                    return;
                }
            };

            let database_service = take_once(&database_service);
            let download_tx = take_once(&download_tx);
            let translation_tx = take_once(&translation_tx);
            let alignment_tx = take_once(&alignment_tx);
            let rx = take_once(&rx);

            let app = Rc::new(RefCell::new(App::new(
                renderer,
                database_service,
                (*mpris_client).clone(),
                download_tx,
                translation_tx,
                alignment_tx,
                &config,
            )));

            app.borrow().renderer.present();

            event_sink(app, rx)
        });

        run_application(&application);

        let _ = shutdown_tx.send(());
    });

    let _ = shutdown_rx.await;

    Ok(())
}
