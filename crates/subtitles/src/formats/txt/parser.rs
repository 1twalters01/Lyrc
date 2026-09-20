use std::str::FromStr;

use uuid::Uuid;

use crate::{
    formats::txt::error::TxtError,
    language::Language,
    parser::SubtitleParser,
    subtitles::{Line, SubtitleCues, SubtitleDocument},
};

enum TxtLine {
    Metadata { key: String, value: String },
    Line { text: String },
    Empty,
}

enum TxtLineType {
    Metadata,
    Line,
    Empty,
}

pub struct TxtParser;

impl SubtitleParser for TxtParser {
    type Error = TxtError;

    fn parse(&self, input: &str) -> Result<SubtitleDocument, Self::Error> {
        let lrc_lines = input
            .lines()
            .map(|line| Self::parse_line(line))
            .collect::<Result<Vec<_>, _>>()?;

        let mut subtitle_document = TxtParser::build_subtitle_document(lrc_lines);

        subtitle_document.update_languages();

        Ok(subtitle_document)
    }
}

impl TxtParser {
    fn parse_line(line: &str) -> Result<TxtLine, TxtError> {
        match Self::get_line_type(line) {
            TxtLineType::Metadata => Self::parse_metadata(line),
            TxtLineType::Line => Ok(TxtLine::Line {
                text: line.to_string(),
            }),
            TxtLineType::Empty => Ok(TxtLine::Empty),
        }
    }

    fn get_line_type(line: &str) -> TxtLineType {
        let line = line.trim();

        if line.is_empty() {
            return TxtLineType::Empty;
        }

        if let Some(tag) = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
        {
            if let Some((key, _)) = tag.split_once(":") {
                if key.len() >= 2 && key.chars().all(|c| c.is_ascii_alphabetic()) {
                    return TxtLineType::Metadata;
                }
            }
        }

        TxtLineType::Line
    }

    fn parse_metadata(line: &str) -> Result<TxtLine, TxtError> {
        let tag = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
            .ok_or(TxtError::InvalidMetadata)?;

        let (key, value) = tag
            .split_once(":")
            .ok_or(TxtError::MissingMetadataSeparator)?;

        Ok(TxtLine::Metadata {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }

    fn build_subtitle_document(lrc_lines: Vec<TxtLine>) -> SubtitleDocument {
        let mut subtitle_document = SubtitleDocument::default();

        for lrc_line in lrc_lines {
            match lrc_line {
                TxtLine::Metadata { key, value } => match key.to_ascii_lowercase().as_str() {
                    "ti" => subtitle_document.metadata.title = Some(value),
                    "al" => subtitle_document.metadata.album = Some(value),
                    "la" => {
                        if let Some(code) = Language::from_str(&value).ok() {
                            subtitle_document.metadata.languages.push(code.into())
                        }
                    }
                    "ar" => subtitle_document
                        .metadata
                        .artists
                        .extend(value.split(',').map(|artist| artist.trim().to_owned())),
                    _ => {}
                },
                TxtLine::Line { text } => {
                    subtitle_document
                        .cues
                        .extend(SubtitleCues::Line(Vec::from([Line {
                            id: Uuid::new_v4(),
                            content: text,
                        }])));
                }
                TxtLine::Empty => {}
            }
        }

        subtitle_document
    }
}
