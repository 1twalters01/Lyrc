use crate::{
    formats::{
        elrc::{parser::ElrcParser, writer::ElrcWriter},
        lrc::{parser::LrcParser, writer::LrcWriter},
        txt::parser::TxtParser,
    },
    language::Language,
    parser::SubtitleParser,
    writer::SubtitleWriter,
};
use chrono::Duration;
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum SyncLevel {
    Phoneme,
    Word,
    Cue,
    None,
}

#[derive(Clone, Debug)]
pub struct SubtitleDocument {
    pub metadata: SubtitleMetadata,
    pub cues: SubtitleCues,
}

impl SubtitleDocument {
    pub fn sync_level(&self) -> SyncLevel {
        match self.cues {
            SubtitleCues::Word(_) => SyncLevel::Word,
            SubtitleCues::Cue(_) => SyncLevel::Cue,
            SubtitleCues::Line(_) => SyncLevel::None,
            SubtitleCues::None => SyncLevel::None,
        }
    }

    pub fn from_pathbuf(path: PathBuf) -> Result<SubtitleDocument, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        match &path.extension() {
            // Make this an actual error type
            None => Err(String::from("File does not have an extension").into()),
            Some(os_str) => match os_str.to_str() {
                Some("elrc") => {
                    let parser = ElrcParser;
                    let mut subtitle_document = parser.parse(&content)?;
                    subtitle_document.metadata.file_path = Some(path);
                    Ok(subtitle_document)
                }
                Some("lrc") => {
                    let parser = LrcParser;
                    let mut subtitle_document = parser.parse(&content)?;
                    subtitle_document.metadata.file_path = Some(path);
                    Ok(subtitle_document)
                }
                Some("txt") => {
                    let parser = TxtParser;
                    let mut subtitle_document = parser.parse(&content)?;
                    subtitle_document.metadata.file_path = Some(path);
                    Ok(subtitle_document)
                }
                Some(_) => Err(String::from("unknown file type").into()),
                None => Err(String::from("os str cannot be turned into a &str").into()),
            },
        }
    }

    pub fn write(
        subtitle_document: &SubtitleDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match &subtitle_document.metadata.file_path {
            Some(file_path) => match file_path.extension() {
                Some(os_str) => match os_str.to_str() {
                    Some("elrc") => {
                        let writer = ElrcWriter;
                        let file = writer.write(&subtitle_document.clone())?;
                        Ok(file)
                    }
                    Some("lrc") => {
                        let writer = LrcWriter;
                        let file = writer.write(&subtitle_document.clone())?;
                        Ok(file)
                    }
                    Some(_) => Err(String::from("unknown file type").into()),
                    None => Err(String::from("os str cannot be turned into a &str").into()),
                },
                None => Err(String::from("File does not have an extension").into()),
            },
            None => Err(String::from("File path does not exist").into()),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.metadata.file_path {
            Some(file_path) => {
                let file = SubtitleDocument::write(self)?;
                Ok(fs::write(file_path, file)?)
            }
            None => Ok(()),
        }
    }

    pub fn update_languages(&mut self) {
        let text = match &self.cues.get_lines() {
            Some(lines) => lines.join("\n"),
            None => return,
        };

        if let Some(info) = whatlang::detect(&text) {
            self.metadata.languages.push(info.lang().into());
        }

        self.metadata.languages.dedup();
    }
}

impl Default for SubtitleDocument {
    fn default() -> Self {
        let metadata = SubtitleMetadata::default();
        let cues = SubtitleCues::default();

        Self { metadata, cues }
    }
}

#[derive(Clone, Debug)]
pub struct SubtitleMetadata {
    pub album: Option<String>,
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub languages: Vec<Language>,
    pub file_path: Option<PathBuf>,
}

impl Default for SubtitleMetadata {
    fn default() -> Self {
        Self {
            album: None,
            title: None,
            artists: Vec::new(),
            languages: Vec::new(),
            file_path: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubtitleCues {
    Word(Vec<AlignedCue>),
    Cue(Vec<Cue>),
    Line(Vec<Line>),
    None,
}

impl Default for SubtitleCues {
    fn default() -> Self {
        Self::None
    }
}

impl SubtitleCues {
    pub fn extend(&mut self, other: Self) {
        if *self == Self::None {
            *self = other;
            return;
        }

        match (self, other) {
            (Self::Word(a), Self::Word(b)) => a.extend(b),
            (Self::Cue(a), Self::Cue(b)) => a.extend(b),
            (Self::Line(a), Self::Line(b)) => a.extend(b),
            _ => {}
        }
    }

    pub fn cue_len(&self, index: usize) -> usize {
        match self {
            Self::Word(words) => words[index].cue_len(),
            Self::Cue(cues) => cues[index].content.len(),
            Self::Line(lines) => lines[index].content.len(),
            Self::None => 0,
        }
    }

    pub fn get_lines(&self) -> Option<Vec<String>> {
        match self {
            SubtitleCues::Word(aligned_cues) => Some(
                aligned_cues
                    .iter()
                    .map(|aligned_cue| {
                        aligned_cue
                            .words
                            .iter()
                            .map(|word| word.content.clone())
                            .collect::<Vec<String>>()
                            .join("\n")
                    })
                    .collect::<Vec<String>>(),
            ),
            SubtitleCues::Cue(cues) => Some(
                cues.iter()
                    .map(|cue| cue.content.clone())
                    .collect::<Vec<String>>(),
            ),
            SubtitleCues::Line(lines) => Some(
                lines
                    .iter()
                    .map(|line| line.content.clone())
                    .collect::<Vec<String>>(),
            ),
            SubtitleCues::None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub id: Uuid,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub id: Uuid,
    pub start: Duration,
    pub end: Duration,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlignedCue {
    pub id: Uuid,
    pub start: Duration,
    pub end: Duration,
    pub words: Vec<Word>,
}

impl AlignedCue {
    pub fn cue_len(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.content.clone())
            .collect::<Vec<String>>()
            .join(" ")
            .len()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub start: Duration,
    pub end: Duration,
    pub content: String,
}
