#[derive(Debug, PartialEq)]
pub struct Lyrics {
    pub content: String,
    pub format: LyricsFormat,
    pub source: LyricsSource,
}

#[derive(Debug, PartialEq)]
pub enum LyricsFormat {
    Lrc,
    Text,
}

#[derive(Debug, PartialEq)]
pub enum LyricsSource {
    Python,
    Lrclib,
}
