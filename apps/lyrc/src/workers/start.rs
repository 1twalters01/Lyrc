use aligner::messages::{AlignmentRequest, AlignmentResult};
use downloader::messages::{DownloadRequest, DownloadResult};
use pyo3::{PyErr, Python};
use tokio::sync::mpsc;
use translator::messages::{TranslationRequest, TranslationResult};

use crate::workers::{
    alignment::AlignmentWorker, download::DownloadWorker,
    translation::TranslationWorker,
};

pub struct WorkersRx {
    pub download_rx: mpsc::Receiver<DownloadResult>,
    pub translation_rx: mpsc::Receiver<TranslationResult>,
    pub alignment_rx: mpsc::Receiver<AlignmentResult>,
}

pub struct WorkersTx {
    pub download_tx: mpsc::Sender<DownloadRequest>,
    pub translation_tx: mpsc::Sender<TranslationRequest>,
    pub alignment_tx: mpsc::Sender<AlignmentRequest>,
}

pub struct Workers {
    pub rx: WorkersRx,
    pub tx: WorkersTx,
}

impl Workers {
    pub async fn start() -> Result<Self, PyErr> {
        let locals = Python::attach(|py| {
            pyo3_async_runtimes::tokio::get_current_locals(py)
        })?;

        let download_worker = DownloadWorker::start(locals.clone()).await;
        let translation_worker = TranslationWorker::start(locals.clone()).await;
        let alignment_worker = AlignmentWorker::start();
        let workers = Self::from_workers(
            download_worker,
            translation_worker,
            alignment_worker,
        );

        Ok(workers)
    }

    fn from_workers(
        download_worker: DownloadWorker,
        translation_worker: TranslationWorker,
        alignment_worker: AlignmentWorker,
    ) -> Workers {
        Self {
            rx: WorkersRx {
                download_rx: download_worker.result_rx,
                translation_rx: translation_worker.result_rx,
                alignment_rx: alignment_worker.result_rx,
            },
            tx: WorkersTx {
                download_tx: download_worker.request_tx,
                translation_tx: translation_worker.request_tx,
                alignment_tx: alignment_worker.request_tx,
            },
        }
    }
}
