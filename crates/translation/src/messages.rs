use subtitles::{language::Language, subtitles::SubtitleDocument};

use crate::error::TranslationError;

pub enum TranslationRequest {
    Translate(TranslationTask),
    // Cancel,
}

#[derive(Debug)]
pub struct TranslationTask {
    pub language: Language,
    pub subtitle_document: SubtitleDocument,
}

// Change error to not be string
#[derive(Debug)]
pub enum TranslationResult {
    Complete(Option<(SubtitleDocument, Language)>),
    Cancelled,
    Failed(TranslationError),
}
