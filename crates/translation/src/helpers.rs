use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::Duration;
use pyo3::{prelude::*, types::PyList};
use subtitles::{
    language::Language,
    subtitles::{Cue, SubtitleCues, SubtitleDocument, SubtitleMetadata},
};
use uuid::Uuid;

use crate::error::TranslationError;

pub fn timedelta_to_duration(timedelta: &Bound<'_, PyAny>) -> PyResult<Duration> {
    let days: i64 = timedelta.getattr("days")?.extract()?;
    let seconds: i64 = timedelta.getattr("seconds")?.extract()?;
    let microseconds: i64 = timedelta.getattr("microseconds")?.extract()?;

    Ok(chrono::Duration::days(days)
        + chrono::Duration::seconds(seconds)
        + chrono::Duration::microseconds(microseconds))
}

pub fn convert_py_cues_to_translated_subtitle_document(
    py_translated_cues: Py<PyAny>,
    subtitle_document: SubtitleDocument,
    language: Language,
) -> Result<Option<SubtitleDocument>, TranslationError> {
    let translated_cues = convert_py_cues_to_cues(py_translated_cues, &subtitle_document)?;

    let translated_metadata = generate_translated_metadata(subtitle_document, language);

    Ok(Some(SubtitleDocument {
        metadata: translated_metadata,
        cues: translated_cues,
    }))
}

pub fn convert_py_cues_to_cues(
    py_translated_cues: Py<PyAny>,
    subtitle_document: &SubtitleDocument,
) -> Result<SubtitleCues, TranslationError> {
    let ids: Vec<Uuid> = match &subtitle_document.cues {
        SubtitleCues::Word(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Cue(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Line(lines) => lines.iter().map(|l| l.id).collect(),
        SubtitleCues::None => return Err(TranslationError::NoSubtitles),
    };

    Python::attach(|py| -> PyResult<SubtitleCues> {
        let binded_py_translated_cues = py_translated_cues.bind(py).cast::<PyList>()?;
        let translated_cues: PyResult<Vec<Cue>> = binded_py_translated_cues
            .iter()
            .enumerate()
            .map(|(i, cue)| {
                let start = timedelta_to_duration(&cue.getattr("start")?)?;
                let end = timedelta_to_duration(&cue.getattr("end")?)?;
                let content = cue.getattr("content")?.extract()?;

                // Need to check that length of subtitle_document.cues
                // is the same as the length of aligned_cues
                Ok::<Cue, PyErr>(Cue {
                    id: ids[i],
                    start,
                    end,
                    content: content,
                })
            })
            .collect();
        Ok(SubtitleCues::Cue(translated_cues?))
    })
    .map_err(|e| TranslationError::PythonError { error: e })
}

pub fn generate_translated_metadata(
    subtitle_document: SubtitleDocument,
    language: Language,
) -> SubtitleMetadata {
    SubtitleMetadata {
        album: subtitle_document.metadata.album,
        title: subtitle_document.metadata.title,
        artists: subtitle_document.metadata.artists,
        languages: subtitle_document.metadata.languages,
        file_path: match subtitle_document.metadata.file_path {
            Some(path) => {
                let new_path = translated_path(&path, &language);
                Some(new_path)
            }
            None => None,
        },
    }
}

fn translated_path(path: &Path, language: &Language) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path.file_stem().unwrap().to_string_lossy();
    let stem = remove_language_suffixes(&stem);
    let filename = format!("{}.{}.lrc", stem, language.as_code_2());
    parent.join(filename)
}

fn remove_language_suffixes(stem: &str) -> &str {
    let mut stem = stem;

    loop {
        let Some((base, suffix)) = stem.rsplit_once(".") else {
            return stem;
        };

        if Language::from_str(suffix).is_ok() {
            stem = base;
        } else {
            return stem;
        }
    }
}
