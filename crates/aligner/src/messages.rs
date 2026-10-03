use std::path::PathBuf;

use subtitles::subtitles::SubtitleDocument;

use crate::error::AlignmentError;

pub enum AlignmentRequest {
    Align(AlignmentTask),
    // Cancel,
}

pub struct AlignmentTask {
    pub audio_file_path: PathBuf,
    pub subtitle_document: SubtitleDocument,
}

// Change error to not be string
#[derive(Debug)]
pub enum AlignmentResult {
    Complete(Option<SubtitleDocument>),
    Cancelled,
    Failed(AlignmentError),
}
