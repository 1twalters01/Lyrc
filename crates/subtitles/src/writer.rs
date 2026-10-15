use crate::subtitles::SubtitleDocument;

pub trait SubtitleWriter {
    type Error: std::error::Error;

    fn write(
        &self,
        subtitle_document: &SubtitleDocument,
    ) -> Result<String, Self::Error>;
}
