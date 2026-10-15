use clap::Parser;
use pyo3::PyResult;

mod args;
mod interfaces;
mod keyboard;
mod run;
mod workers;

#[pyo3_async_runtimes::tokio::main]
async fn main() -> PyResult<()> {
    let args = args::Args::parse();
    run::run(args).await.map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
    })?;

    Ok(())
}
