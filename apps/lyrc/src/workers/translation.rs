use pyo3_async_runtimes::TaskLocals;
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
    pub async fn start(locals: TaskLocals) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<TranslationRequest>(1);
        let (result_tx, result_rx) = mpsc::channel::<TranslationResult>(1);
        Self::handle(request_rx, result_tx, locals);

        Self {
            request_tx,
            result_rx,
        }
    }

    fn handle(
        mut request_rx: mpsc::Receiver<TranslationRequest>,
        result_tx: mpsc::Sender<TranslationResult>,
        locals: TaskLocals,
    ) {
        println!("start handling of translation");
        tokio::spawn(async move {
            println!("Inside of tokio spawn");
            while let Some(request) = request_rx.recv().await {
                match request {
                    TranslationRequest::Translate(TranslationTask {
                        language,
                        subtitle_document,
                    }) => {
                        let argos_result = ArgosTranslator
                            .translate(language, subtitle_document, locals.clone())
                            .await;

                        let result = match argos_result {
                            Ok(Some(document)) => {
                                TranslationResult::Complete(Some((document, language)))
                            }
                            Ok(None) => TranslationResult::Complete(None),
                            Err(error) => TranslationResult::Failed(error),
                        };

                        if result_tx.send(result).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }
}
