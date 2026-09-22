use std::thread;

use tokio::sync::mpsc;
use translation::{
    messages::{TranslationRequest, TranslationResult, TranslationTask},
    provider::LyricsTranslator,
    providers::argos::ArgosTranslator,
};

pub struct TranslationWorker {
    pub request_tx: mpsc::Sender<TranslationRequest>,
    pub result_rx: mpsc::Receiver<TranslationResult>,
}

impl TranslationWorker {
    pub async fn start() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<TranslationRequest>(1);
        let (result_tx, result_rx) = mpsc::channel::<TranslationResult>(1);
        Self::handle(request_rx, result_tx).await;

        Self {
            request_tx,
            result_rx,
        }
    }

    async fn handle(
        mut request_rx: mpsc::Receiver<TranslationRequest>,
        result_tx: mpsc::Sender<TranslationResult>,
    ) {
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                while let Some(request) = request_rx.recv().await {
                    match request {
                        TranslationRequest::Translate(TranslationTask {
                            language,
                            subtitle_document,
                        }) => {
                            let argos_result = ArgosTranslator.translate(language, subtitle_document);

                            let result = match argos_result.await {
                                Ok(subtitle_document) => match subtitle_document {
                                    Some(document) => TranslationResult::Complete(Some((document, language))),
                                    None => TranslationResult::Complete(None)
                                }
                                Err(error) => TranslationResult::Failed(error),
                            };

                            // decide to use sync or async send
                            // if result_tx.blocking_send(result).is_err() {
                            if result_tx.send(result).await.is_err() {
                                break;
                            }
                        }
                        // TranslationRequest::Cancel => {
                        // * }, 
                    }
                }
            })
        });
    }
}
