use std::{cell::RefCell, rc::Rc};

use crate::workers::start::Workers;

use gtk::{gio::prelude::*, glib::MainContext};

use configuration::config::Config;
use database::service::DatabaseService;
use gui::{
    application::{build_application, run_application, setup_application},
    renderer::GuiRenderer,
};
use lyrc_core::app::App;
use mpris::client::MprisClient;

struct GuiInit {
    config: Config,
    database_service: DatabaseService,
    mpris_client: Option<MprisClient>,
    workers: Workers,
}

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

    run_app(
        database_service,
        mpris_client,
        workers,
        config,
    ).await
}

pub async fn run_app(
    database_service: DatabaseService,
    mpris_client: Option<MprisClient>,
    workers: Workers,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let application = build_application(&config.application_id);

    let init = Rc::new(RefCell::new(Some(GuiInit {
        config,
        database_service,
        mpris_client,
        workers,
    })));

    application.connect_activate(move |application| {
        let Some(init) = init.borrow_mut().take() else {
            return;
        };

        let renderer = match GuiRenderer::new(application) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Gui renderer error: {error}");
                return;
            }
        };

        renderer.present();

        MainContext::default().spawn_local(async move {
            let GuiInit {
                config,
                database_service,
                mpris_client,
                workers,
            } = init;

            let mut app = App::new(
                renderer,
                database_service,
                mpris_client,
                workers.tx.download_tx,
                workers.tx.translation_tx,
                workers.tx.alignment_tx,
                &config,
            );

            app.update().await;
        });
    });

    run_application(&application);

    Ok(())
}
