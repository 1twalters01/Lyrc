use gtk::{Application, gio, prelude::*};

pub fn build_application(application_id: &str) -> Application {
    Application::builder()
        .application_id(application_id)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build()
}

pub fn run_application(application: &Application) {
    let argv0 = std::env::args()
        .next()
        .unwrap_or_else(|| "lyrc".to_string());

    application.run_with_args(&[argv0.as_str()]);
}
