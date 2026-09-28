use std::str::FromStr;

use chrono::Duration;
use uuid::Uuid;

use crate::{
    formats::elrc::error::ElrcError,
    language::Language,
    parser::SubtitleParser,
    subtitles::{AlignedCue, SubtitleCues, SubtitleDocument, Word},
};

#[derive(Debug, Clone)]
struct ElrcWord {
    timestamp: Duration,
    content: String,
}

#[derive(Debug, Clone)]
enum ElrcLine {
    Metadata {
        key: String,
        value: String,
    },

    Lyric {
        timestamps: Vec<Duration>,
        words: Vec<ElrcWord>,
    },
    Empty,
    Unknown {
        value: String,
    },
}

enum ElrcLineType {
    Metadata,
    Lyric,
    Empty,
    Unknown,
}

pub struct ElrcParser;

impl SubtitleParser for ElrcParser {
    type Error = ElrcError;

    fn parse(&self, input: &str) -> Result<SubtitleDocument, Self::Error> {
        let lrc_lines = input
            .lines()
            .map(|line| Self::parse_line(line))
            .collect::<Result<Vec<_>, _>>()?;

        let mut subtitle_document = ElrcParser::build_subtitle_document(lrc_lines);

        subtitle_document.update_languages();

        Ok(subtitle_document)
    }
}

impl ElrcParser {
    fn parse_line(line: &str) -> Result<ElrcLine, ElrcError> {
        match Self::get_line_type(line) {
            ElrcLineType::Metadata => Self::parse_metadata(line),
            ElrcLineType::Lyric => Self::parse_lyric(line),
            ElrcLineType::Empty => Ok(ElrcLine::Empty),
            ElrcLineType::Unknown => Ok(ElrcLine::Unknown { value: line.into() }),
        }
    }

    fn get_line_type(line: &str) -> ElrcLineType {
        let line = line.trim();

        if line.is_empty() {
            return ElrcLineType::Empty;
        }

        let Some(tag) = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
        else {
            return ElrcLineType::Unknown;
        };

        if Self::parse_timestamp(tag).is_ok() {
            return ElrcLineType::Lyric;
        }

        if let Some((key, _)) = tag.split_once(":") {
            if key.len() >= 2 && key.chars().all(|c| c.is_ascii_alphabetic()) {
                return ElrcLineType::Metadata;
            }
        }

        ElrcLineType::Unknown
    }

    fn parse_metadata(line: &str) -> Result<ElrcLine, ElrcError> {
        let tag = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(tag, _)| tag)
            .ok_or(ElrcError::InvalidMetadata)?;

        let (key, value) = tag
            .split_once(":")
            .ok_or(ElrcError::MissingMetadataSeparator)?;

        Ok(ElrcLine::Metadata {
            key: String::from(key),
            value: String::from(value),
        })
    }

    fn parse_timestamp(input: &str) -> Result<Duration, ElrcError> {
        let trimmed_input = input.trim_start_matches('[').trim_end_matches(']');

        let colon_idx = match trimmed_input.find(':') {
            Some(colon_idx) => colon_idx,
            None => return Err(ElrcError::MissingColonSeparatorInTimestamp),
        };

        let minutes = match trimmed_input[..colon_idx].parse::<i64>() {
            Ok(minutes) => Duration::minutes(minutes),
            Err(_) => return Err(ElrcError::InvalidTimestamp),
        };

        let remainder = &trimmed_input[colon_idx + 1..];
        let (seconds, milliseconds) = match remainder.find('.') {
            Some(dot_idx) => {
                let seconds = match remainder[..dot_idx].parse::<i64>() {
                    Ok(seconds) => Duration::seconds(seconds),
                    Err(_) => return Err(ElrcError::InvalidTimestamp),
                };

                let milliseconds_str = &remainder[dot_idx + 1..];
                let milliseconds = match milliseconds_str.len() {
                    2 => match milliseconds_str.parse::<i64>() {
                        Ok(milliseconds) => Duration::milliseconds(milliseconds * 10),
                        Err(_) => return Err(ElrcError::InvalidTimestamp),
                    },
                    3 => match milliseconds_str.parse::<i64>() {
                        Ok(milliseconds) => Duration::milliseconds(milliseconds),
                        Err(_) => return Err(ElrcError::InvalidTimestamp),
                    },
                    _ => {
                        return Err(ElrcError::InvalidTimestampMillisecondFormat);
                    }
                };

                (seconds, milliseconds)
            }
            None => {
                let seconds = match remainder.parse::<i64>() {
                    Ok(seconds) => Duration::seconds(seconds),
                    Err(_) => return Err(ElrcError::InvalidTimestamp),
                };
                let milliseconds = Duration::milliseconds(0);
                (seconds, milliseconds)
            }
        };

        Ok(minutes + seconds + milliseconds)
    }

    fn parse_lyric(line: &str) -> Result<ElrcLine, ElrcError> {
        let mut remaining_line = line;

        let mut timestamps = Vec::new();
        let mut words = Vec::new();

        while let Some(stripped_line) = remaining_line.strip_prefix('[') {
            let Some(end) = stripped_line.find(']') else {
                return Err(ElrcError::MissingTagClosingBracket);
            };

            let tag = &stripped_line[..end];
            remaining_line = &stripped_line[end + 1..];

            timestamps.push(ElrcParser::parse_timestamp(tag)?);
        }

        let remaining_line = remaining_line
            .strip_prefix(" ")
            .unwrap_or(remaining_line);

        for word in remaining_line.split(" ") {
            let mut remaining_word = word;
            let mut word_timestamps = Vec::new();

            while let Some(stripped_word) = remaining_word.strip_prefix('<') {
                let Some(end) = stripped_word.find('>') else {
                    return Err(ElrcError::MissingWordTimeClosingBracket);
                };

                let word_timestamp = &stripped_word[..end];
                remaining_word = &stripped_word[end + 1..];

                word_timestamps.push(ElrcParser::parse_timestamp(word_timestamp)?);
            }

            for timestamp in word_timestamps {
                words.push(ElrcWord {
                    timestamp,
                    content: String::from(remaining_word),
                });
            }
        }

        Ok(ElrcLine::Lyric { timestamps, words })
    }

    fn add_metadata(subtitle_document: &mut SubtitleDocument, key: &str, value: &str) {
        match key.to_ascii_lowercase().as_str() {
            "ti" => subtitle_document.metadata.title = Some(String::from(value)),
            "al" => subtitle_document.metadata.album = Some(String::from(value)),
            "la" => {
                subtitle_document.metadata.languages.extend(
                    value
                        .split(',')
                        .filter_map(|language| Language::from_str(language.trim()).ok()),
                );
            }
            "ar" => subtitle_document
                .metadata
                .artists
                .extend(value.split(',').map(|artist| artist.trim().to_owned())),
            _ => {}
        }
    }

    fn next_lyric_timestamp(lines: &Vec<ElrcLine>, current_index: usize) -> Option<Duration> {
        lines
            .iter()
            .skip(current_index + 1)
            .find_map(|line| match line {
                ElrcLine::Lyric {
                    timestamps,
                    words: _,
                } => timestamps.first().copied(),
                _ => None,
            })
    }

    fn build_aligned_words(
        words: &Vec<ElrcWord>,
        next_lyric_timestamp: Option<Duration>,
    ) -> Vec<Word> {
        words
            .iter()
            .enumerate()
            .map(|(index, word)| {
                let end = words
                    .get(index + 1)
                    .map(|next| next.timestamp)
                    .or(next_lyric_timestamp)
                    .unwrap_or(word.timestamp);

                Word {
                    start: word.timestamp,
                    end,
                    content: word.content.clone(),
                }
            })
            .collect()
    }

    fn build_subtitle_document(lines: Vec<ElrcLine>) -> SubtitleDocument {
        let mut subtitle_document = SubtitleDocument::default();

        for (line_index, line) in lines.iter().enumerate() {
            match line {
                ElrcLine::Metadata { key, value } => {
                    ElrcParser::add_metadata(&mut subtitle_document, key, value)
                }
                ElrcLine::Lyric { timestamps, words } => {
                    let next_lyric_timestamp = Self::next_lyric_timestamp(&lines, line_index);

                    let aligned_words = Self::build_aligned_words(words, next_lyric_timestamp);

                    subtitle_document.cues.extend(SubtitleCues::Word(
                        timestamps
                            .iter()
                            .map(|timestamp| AlignedCue {
                                id: Uuid::new_v4(),
                                start: *timestamp,
                                end: aligned_words
                                    .last()
                                    .map(|word| word.end)
                                    .unwrap_or(*timestamp),
                                words: aligned_words.clone(),
                            })
                            .collect(),
                    ));
                }
                ElrcLine::Empty => {}
                ElrcLine::Unknown { value } => {}
            }
        }

        if let SubtitleCues::Word(ref mut cues) = subtitle_document.cues {
            cues.sort_by_key(|c| c.start);
        }

        subtitle_document
    }
}
