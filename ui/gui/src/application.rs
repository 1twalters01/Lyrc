use gtk::{Application, ApplicationWindow, gio, prelude::*};

pub fn build_application(application_id: &str) -> Application {
    Application::builder()
        .application_id(application_id)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build()
}

pub fn setup_application(application: &Application) {
    application.connect_activate(|application| {
        let window = ApplicationWindow::builder()
            .application(application)
            .title("Lyrc")
            .build();

        window.present();
    });
}

pub fn run_application(application: &Application) {
    let argv0 = std::env::args()
        .next()
        .unwrap_or_else(|| "lyrc".to_string());

    application.run_with_args(&[argv0.as_str()]);
}
