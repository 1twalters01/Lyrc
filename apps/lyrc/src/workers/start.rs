use aligner::messages::{AlignmentRequest, AlignmentResult};
use pyo3::{PyErr, Python};
use tokio::sync::mpsc;
use translator::messages::{TranslationRequest, TranslationResult};

use crate::workers::{alignment::AlignmentWorker, translation::TranslationWorker};

pub struct WorkersRx {
    pub alignment_rx: mpsc::Receiver<AlignmentResult>,
    pub translation_rx: mpsc::Receiver<TranslationResult>,
}

pub struct WorkersTx {
    pub alignment_tx: mpsc::Sender<AlignmentRequest>,
    pub translation_tx: mpsc::Sender<TranslationRequest>,
}

pub struct Workers {
    pub rx: WorkersRx,
    pub tx: WorkersTx,
}

impl Workers {
    pub async fn start() -> Result<Self, PyErr> {
        let locals = Python::attach(|py| pyo3_async_runtimes::tokio::get_current_locals(py))?;

        let alignment_worker = AlignmentWorker::start();
        let translation_worker = TranslationWorker::start(locals).await;
        let workers = Self::from_workers(alignment_worker, translation_worker);

        Ok(workers)
    }

    fn from_workers(
        alignment_worker: AlignmentWorker,
        translation_worker: TranslationWorker,
    ) -> Workers {
        Self {
            rx: WorkersRx {
                alignment_rx: alignment_worker.result_rx,
                translation_rx: translation_worker.result_rx,
            },
            tx: WorkersTx {
                alignment_tx: alignment_worker.request_tx,
                translation_tx: translation_worker.request_tx,
            },
        }
    }
}
