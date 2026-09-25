use pyo3::Python;

use crate::workers::{alignment::AlignmentWorker, translation::TranslationWorker};

pub struct Workers {
    pub alignment: AlignmentWorker,
    pub translation: TranslationWorker,
}

impl Workers {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let locals = Python::attach(|py| pyo3_async_runtimes::tokio::get_current_locals(py))?;

        let alignment = AlignmentWorker::start();
        let translation = TranslationWorker::start(locals).await;

        Ok(Self {
            alignment,
            translation,
        })
    }
}
