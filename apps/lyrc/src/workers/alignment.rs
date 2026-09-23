use std::thread;

use alignment::{
    messages::{AlignmentRequest, AlignmentResult, AlignmentTask},
    provider::LyricsAligner,
    providers::{whisperx_cue::WhisperXCueAligner, whisperx_word::WhisperXWordAligner},
};
use subtitles::subtitles::SyncLevel;
use tokio::sync::mpsc;

pub struct AlignmentWorker {
    pub result_rx: mpsc::Receiver<AlignmentResult>,
    pub request_tx: mpsc::Sender<AlignmentRequest>,
}

impl AlignmentWorker {
    pub fn start() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<AlignmentRequest>(1);
        let (result_tx, result_rx) = mpsc::channel::<AlignmentResult>(1);
        Self::handle(request_rx, result_tx);

        Self {
            request_tx,
            result_rx,
        }
    }

    pub fn handle(
        mut request_rx: tokio::sync::mpsc::Receiver<AlignmentRequest>,
        result_tx: tokio::sync::mpsc::Sender<AlignmentResult>,
    ) {
        thread::spawn(move || {
            while let Some(request) = request_rx.blocking_recv() {
                match request {
                    AlignmentRequest::Align(AlignmentTask {
                        audio_file_path,
                        subtitle_document,
                    }) => {
                        let alignment_result = match subtitle_document.sync_level() {
                            SyncLevel::None => WhisperXCueAligner::align_cues(
                                audio_file_path,
                                subtitle_document
                            ),
                            SyncLevel::Cue => WhisperXWordAligner::align_cues(
                                audio_file_path,
                                subtitle_document
                            ),
                            SyncLevel::Word => return,
                            SyncLevel::Phoneme => return,
                        };


                        let result = match alignment_result {
                            Ok(subtitle_document) => AlignmentResult::Complete(subtitle_document),
                            Err(error) => AlignmentResult::Failed(error),
                        };
                        // println!("\ndone");

                        if result_tx.blocking_send(result).is_err() {
                            break;
                        }
                    }
                    // AlignmentRequest::Cancel => {
                    //* },
                }
            }
        });
    }
}
