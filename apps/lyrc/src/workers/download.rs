use downloader::{
    messages::{DownloadRequest, DownloadResult, DownloadTask},
    models::LyricsFormat,
    provider::LyricsDownloader,
    providers::lrclib::LrclibProvider,
};
use pyo3_async_runtimes::TaskLocals;
use subtitles::{formats::lrc::parser::LrcParser, parser::SubtitleParser};
use tokio::sync::mpsc;

pub struct DownloadWorker {
    pub request_tx: mpsc::Sender<DownloadRequest>,
    pub result_rx: mpsc::Receiver<DownloadResult>,
}

impl DownloadWorker {
    pub async fn start(locals: TaskLocals) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<DownloadRequest>(1);
        let (result_tx, result_rx) = mpsc::channel::<DownloadResult>(1);
        Self::handle(request_rx, result_tx, locals);

        Self {
            request_tx,
            result_rx,
        }
    }

    fn handle(
        mut request_rx: mpsc::Receiver<DownloadRequest>,
        result_tx: mpsc::Sender<DownloadResult>,
        locals: TaskLocals,
    ) {
        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                match request {
                    DownloadRequest::Download(DownloadTask {
                        track,
                        download_service,
                    }) => {
                        let lyrics_provider = download_service.get_provider();
                        let lyrics_result = lyrics_provider.search(track.clone(), locals.clone()).await;

                        let result = match lyrics_result {
                            Ok(Some(lyrics)) => {
                                let document = match lyrics.format {
                                    LyricsFormat::Lrc => {
                                        let mut document_path = None;
                                        if let Some(file_path) = track.file_path {
                                            let mut lrc_path = file_path.to_path_buf();
                                            lrc_path.set_extension("lrc");
                                            document_path = Some(lrc_path);
                                        }
                                        let document = LrcParser.parse(&lyrics.content);
                                        if let Ok(mut document) = document {
                                            document.metadata.file_path = document_path;
                                            if document.metadata.artists.is_empty() {
                                                document.metadata.artists.extend(track.artists);
                                            }
                                            if document.metadata.title.is_none() {
                                                document.metadata.title = Some(track.title);
                                            }

                                            Some(document)
                                        } else {
                                            None
                                        }
                                    }
                                    LyricsFormat::Text => None,
                                };

                                DownloadResult::Complete(document)
                            }
                            Ok(None) => DownloadResult::Complete(None),
                            Err(error) => DownloadResult::Failed(error),
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
