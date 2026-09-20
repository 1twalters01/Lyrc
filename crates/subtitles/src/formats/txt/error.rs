use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TxtError {
    InvalidSubtitleDocumentFormat,
    MissingTagClosingBracket,
    ContentAfterMetadataTag,
    InvalidMetadata,
    MissingMetadataSeparator,
}

// Make this better later - have line numbers for example
impl fmt::Display for TxtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxtError::InvalidSubtitleDocumentFormat => {
                write!(f, "Invalid subtitle document format")
            }
            TxtError::MissingTagClosingBracket => {
                write!(f, "Missing tag closing bracket")
            }
            TxtError::ContentAfterMetadataTag => {
                write!(f, "Content after metadata tag")
            }
            TxtError::InvalidMetadata => write!(f, "Invalid metadata"),
            TxtError::MissingMetadataSeparator => {
                write!(f, "Missing metadata separator")
            }
        }
    }
}

impl Error for TxtError {}
