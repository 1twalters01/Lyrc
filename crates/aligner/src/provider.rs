use std::path::PathBuf;

use subtitles::subtitles::SubtitleDocument;

use crate::error::AlignmentError;

pub trait LyricsAligner: Send + Sync {
    fn align_cues(
        audio_file_path: PathBuf,
        subtitle_document: SubtitleDocument,
    ) -> Result<Option<SubtitleDocument>, AlignmentError>;
}
