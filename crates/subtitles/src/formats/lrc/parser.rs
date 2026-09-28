use std::str::FromStr;

use chrono::Duration;
use uuid::Uuid;

use crate::{
    formats::lrc::error::LrcError,
    language::Language,
    parser::SubtitleParser,
    subtitles::{Cue, SubtitleCues, SubtitleDocument},
};

enum LrcLine {
    Metadata {
        key: String,
        value: String,
    },
    Lyric {
        timestamps: Vec<Duration>,
        text: String,
    },
    Empty,
    Unknown {
        value: String,
    },
}

enum LrcLineType {
    Metadata,
    Lyric,
    Empty,
    Unknown,
}

pub struct LrcParser;

impl SubtitleParser for LrcParser {
    type Error = LrcError;

    fn parse(&self, input: &str) -> Result<SubtitleDocument, Self::Error> {
        let lrc_lines = input
            .lines()
            .map(|line| Self::parse_line(line))
            .collect::<Result<Vec<_>, _>>()?;

        let mut subtitle_document = LrcParser::build_subtitle_document(lrc_lines);

        subtitle_document.update_languages();

        Ok(subtitle_document)
    }
}

impl LrcParser {
    fn parse_line(line: &str) -> Result<LrcLine, LrcError> {
        match Self::get_line_type(line) {
            LrcLineType::Metadata => Self::parse_metadata(line),
            LrcLineType::Lyric => Self::parse_lyric(line),
            LrcLineType::Empty => Ok(LrcLine::Empty),
            LrcLineType::Unknown => Ok(LrcLine::Unknown { value: line.into() }),
        }
    }

    fn get_line_type(line: &str) -> LrcLineType {
        let line = line.trim();

        if line.is_empty() {
            return LrcLineType::Empty;
        }

        let Some(tag) = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
        else {
            return LrcLineType::Unknown;
        };

        if Self::parse_timestamp(tag).is_ok() {
            return LrcLineType::Lyric;
        }

        if let Some((key, _)) = tag.split_once(":") {
            if key.len() >= 2 && key.chars().all(|c| c.is_ascii_alphabetic()) {
                return LrcLineType::Metadata;
            }
        }

        LrcLineType::Unknown
    }

    fn parse_metadata(line: &str) -> Result<LrcLine, LrcError> {
        let tag = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
            .ok_or(LrcError::InvalidMetadata)?;

        let (key, value) = tag
            .split_once(":")
            .ok_or(LrcError::MissingMetadataSeparator)?;

        Ok(LrcLine::Metadata {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }

    fn parse_timestamp(input: &str) -> Result<Duration, LrcError> {
        let trimmed_input = input.trim_start_matches('[').trim_end_matches(']');

        let colon_idx = match trimmed_input.find(':') {
            Some(colon_idx) => colon_idx,
            None => return Err(LrcError::MissingColonSeparatorInTimestamp),
        };

        let minutes = match trimmed_input[..colon_idx].parse::<i64>() {
            Ok(minutes) => Duration::minutes(minutes),
            Err(_) => return Err(LrcError::InvalidTimestamp),
        };

        let remainder = &trimmed_input[colon_idx + 1..];
        let (seconds, milliseconds) = match remainder.find('.') {
            Some(dot_idx) => {
                let seconds = match remainder[..dot_idx].parse::<i64>() {
                    Ok(seconds) => Duration::seconds(seconds),
                    Err(_) => return Err(LrcError::InvalidTimestamp),
                };

                let milliseconds_str = &remainder[dot_idx + 1..];
                let milliseconds = match milliseconds_str.len() {
                    2 => match milliseconds_str.parse::<i64>() {
                        Ok(milliseconds) => Duration::milliseconds(milliseconds * 10),
                        Err(_) => return Err(LrcError::InvalidTimestamp),
                    },
                    3 => match milliseconds_str.parse::<i64>() {
                        Ok(milliseconds) => Duration::milliseconds(milliseconds),
                        Err(_) => return Err(LrcError::InvalidTimestamp),
                    },
                    _ => {
                        return Err(LrcError::InvalidTimestampMillisecondFormat);
                    }
                };

                (seconds, milliseconds)
            }
            None => {
                let seconds = match remainder.parse::<i64>() {
                    Ok(seconds) => Duration::seconds(seconds),
                    Err(_) => return Err(LrcError::InvalidTimestamp),
                };
                let milliseconds = Duration::milliseconds(0);
                (seconds, milliseconds)
            }
        };

        Ok(minutes + seconds + milliseconds)
    }

    fn parse_lyric(line: &str) -> Result<LrcLine, LrcError> {
        let mut remaining_line = line;
        let mut timestamps = Vec::new();

        while let Some(stripped_line) = remaining_line.strip_prefix('[') {
            let Some(end) = stripped_line.find(']') else {
                return Err(LrcError::MissingTagClosingBracket);
            };

            let tag = &stripped_line[..end];
            remaining_line = &stripped_line[end + 1..];

            timestamps.push(LrcParser::parse_timestamp(tag)?);
        }

        Ok(LrcLine::Lyric {
            timestamps,
            text: remaining_line.trim_start().to_owned(),
        })
    }

    fn build_subtitle_document(lrc_lines: Vec<LrcLine>) -> SubtitleDocument {
        let mut subtitle_document = SubtitleDocument::default();

        for lrc_line in lrc_lines {
            match lrc_line {
                LrcLine::Metadata { key, value } => match key.to_ascii_lowercase().as_str() {
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
                LrcLine::Lyric { timestamps, text } => {
                    let cues = SubtitleCues::Cue(
                        timestamps
                            .into_iter()
                            .map(|timestamp| Cue {
                                id: Uuid::new_v4(),
                                start: timestamp,
                                end: timestamp,
                                content: text.clone(),
                            })
                            .collect(),
                    );
                    subtitle_document.cues.extend(cues);
                }
                LrcLine::Empty => {}
                LrcLine::Unknown { value } => {}
            }
        }

        if let SubtitleCues::Cue(ref mut cues) = subtitle_document.cues {
            cues.sort_by_key(|c| c.start);

            for i in 0..cues.len().saturating_sub(1) {
                let start = cues[i].start;

                if let Some(next) = cues[i + 1..].iter().find(|cue| cue.start > start) {
                    cues[i].end = next.start;
                }
            }
        }

        subtitle_document
    }
}
