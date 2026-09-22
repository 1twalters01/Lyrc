use alignment::{
    error::AlignmentError,
    messages::{AlignmentRequest, AlignmentTask},
};

use crate::{app::App, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn start_alignment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // also check if it is for the same audio/subtitles?
        // if so then store alignment sub/audio in app state?
        // maybe make a struct for alignment status in state?
        if self.state.alignment_running {
            return Ok(());
        }

        let (track, subtitle_document) =
            match (&self.state.track, &self.state.subtitle_documents.active()) {
                (Some(track), Some(subtitle_document_state)) => {
                    (track, subtitle_document_state.document.clone())
                }
                (None, _) => return Err(Box::new(AlignmentError::NoAudioFilePath)),
                (_, None) => return Err(Box::new(AlignmentError::NoSubtitles)),
            };

        let audio_file_path = match &track.file_path {
            Some(path) => path,
            None => return Err(Box::new(AlignmentError::NoAudioFilePath)),
        };

        let task = AlignmentTask {
            audio_file_path: audio_file_path.clone(),
            subtitle_document: subtitle_document.clone(),
        };

        self.alignment_req_tx
            .send(AlignmentRequest::Align(task))
            .await?;

        self.state.alignment_running = true;

        Ok(())
    }
}
