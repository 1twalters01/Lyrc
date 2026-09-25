use futures::future::BoxFuture;
use pyo3_async_runtimes::TaskLocals;
use subtitles::{language::Language, subtitles::SubtitleDocument};

use crate::error::TranslationError;

pub trait LyricsTranslator: Send + Sync {
    fn translate(
        &self,
        language: Language,
        subtitle_document: SubtitleDocument,
        locals: TaskLocals,
    ) -> BoxFuture<'_, Result<Option<SubtitleDocument>, TranslationError>>;
}
