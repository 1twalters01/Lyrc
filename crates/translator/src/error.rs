use std::{error::Error, fmt};

use pyo3::PyErr;

// use thiserror?
#[derive(Debug)]
pub enum TranslationError {
    PythonError { error: PyErr },
    InvalidLanguage,
    NoLanguageCode,
    NoSubtitles,
    AlreadyTranslated,
}

impl From<PyErr> for TranslationError {
    fn from(error: PyErr) -> Self {
        TranslationError::PythonError { error }
    }
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslationError::PythonError { error } => {
                write!(f, "Python error: {error}")
            }
            TranslationError::InvalidLanguage => {
                write!(f, "Invalid language provided")
            }
            TranslationError::NoLanguageCode => {
                write!(f, "No language code found")
            }
            TranslationError::NoSubtitles => {
                write!(f, "No subtitles found for current song")
            }
            TranslationError::AlreadyTranslated => {
                write!(f, "Content is already translated")
            }
        }
    }
}

impl Error for TranslationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TranslationError::PythonError { error } => Some(error),
            TranslationError::InvalidLanguage => None,
            TranslationError::NoSubtitles => None,
            TranslationError::NoLanguageCode => None,
            TranslationError::AlreadyTranslated => None,
        }
    }
}
