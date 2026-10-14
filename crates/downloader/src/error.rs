use pyo3::PyErr;

use std::{error::Error, fmt};

// use thiserror?
#[derive(Debug)]
pub enum DownloadError {
    PythonError { error: PyErr },
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::PythonError { error } => {
                write!(f, "Python error: {error}")
            }
        }
    }
}

impl Error for DownloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DownloadError::PythonError { error } => Some(error),
        }
    }
}
