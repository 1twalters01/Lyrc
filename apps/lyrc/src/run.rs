use configuration::load::load_config;

use crate::{
    args::{Args, Command, Frontend},
    interfaces::{daemon::run_daemon, gui::run::run_gui, tui::run::run_tui},
};

pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = None;
    let json_path = None;
    let config = load_config(toml_path, json_path)?;

    let command = args.command.unwrap_or(Command::App {
        frontend: Frontend::Tui,
    });

    match command {
        Command::Daemon => run_daemon(config).await,
        Command::App { frontend } => match frontend {
            Frontend::Tui => run_tui(config).await,
            Frontend::Gui => run_gui(config).await,
        },
    }
}
