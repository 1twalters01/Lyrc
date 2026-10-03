use chrono::Duration;
use pyo3::{prelude::*, types::PyList};
use subtitles::subtitles::{
    AlignedCue, Cue, SubtitleCues, SubtitleDocument, SubtitleMetadata, Word,
};
use uuid::Uuid;

use crate::error::AlignmentError;

pub fn timedelta_to_duration(timedelta: &Bound<'_, PyAny>) -> PyResult<Duration> {
    let days: i64 = timedelta.getattr("days")?.extract()?;
    let seconds: i64 = timedelta.getattr("seconds")?.extract()?;
    let microseconds: i64 = timedelta.getattr("microseconds")?.extract()?;

    Ok(chrono::Duration::days(days)
        + chrono::Duration::seconds(seconds)
        + chrono::Duration::microseconds(microseconds))
}

pub fn convert_py_cues_to_line_aligned_subtitle_document(
    py_aligned_cues: Py<PyAny>,
    subtitle_document: SubtitleDocument,
) -> Result<Option<SubtitleDocument>, AlignmentError> {
    let aligned_cues = convert_py_cues_to_line_level_cues(py_aligned_cues, &subtitle_document)?;

    let aligned_metadata = generate_aligned_metadata(subtitle_document);

    Ok(Some(SubtitleDocument {
        metadata: aligned_metadata,
        cues: aligned_cues,
    }))
}

pub fn convert_py_cues_to_word_aligned_subtitle_document(
    py_aligned_cues: Py<PyAny>,
    subtitle_document: SubtitleDocument,
) -> Result<Option<SubtitleDocument>, AlignmentError> {
    let aligned_cues = convert_py_cues_to_word_level_cues(py_aligned_cues, &subtitle_document)?;

    let aligned_metadata = generate_aligned_metadata(subtitle_document);

    Ok(Some(SubtitleDocument {
        metadata: aligned_metadata,
        cues: aligned_cues,
    }))
}

pub fn convert_py_cues_to_word_level_cues(
    py_aligned_cues: Py<PyAny>,
    subtitle_document: &SubtitleDocument,
) -> Result<SubtitleCues, AlignmentError> {
    let ids: Vec<Uuid> = match &subtitle_document.cues {
        SubtitleCues::Word(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Cue(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Line(lines) => lines.iter().map(|l| l.id).collect(),
        SubtitleCues::None => return Err(AlignmentError::NoSubtitles),
    };

    Python::attach(|py| -> PyResult<SubtitleCues> {
        let binded_py_aligned_cues = py_aligned_cues.bind(py).cast::<PyList>()?;
        let aligned_cues: PyResult<Vec<AlignedCue>> = binded_py_aligned_cues
            .iter()
            .enumerate()
            .map(|(i, cue)| {
                let start = timedelta_to_duration(&cue.getattr("start")?)?;
                let end = timedelta_to_duration(&cue.getattr("end")?)?;
                let words = cue
                    .getattr("words")?
                    .cast::<PyList>()?
                    .iter()
                    .map(|word| {
                        Ok(Word {
                            start: timedelta_to_duration(&word.getattr("start")?)?,
                            end: timedelta_to_duration(&word.getattr("end")?)?,
                            content: word.getattr("text")?.extract()?,
                        })
                    })
                    .collect::<PyResult<Vec<_>>>()?;

                // Need to check that length of subtitle_document.cues
                // is the same as the length of aligned_cues
                Ok(AlignedCue {
                    id: ids[i],
                    start,
                    end,
                    words: words,
                })
            })
            .collect();
        Ok(SubtitleCues::Word(aligned_cues?))
    })
    .map_err(|e| AlignmentError::PythonError { error: e })
}

pub fn convert_py_cues_to_line_level_cues(
    py_aligned_cues: Py<PyAny>,
    subtitle_document: &SubtitleDocument,
) -> Result<SubtitleCues, AlignmentError> {
    let ids: Vec<Uuid> = match &subtitle_document.cues {
        SubtitleCues::Word(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Cue(cues) => cues.iter().map(|c| c.id).collect(),
        SubtitleCues::Line(lines) => lines.iter().map(|l| l.id).collect(),
        SubtitleCues::None => return Err(AlignmentError::NoSubtitles),
    };

    Python::attach(|py| -> PyResult<SubtitleCues> {
        let binded_py_aligned_cues = py_aligned_cues.bind(py).cast::<PyList>()?;
        let aligned_cues: PyResult<Vec<Cue>> = binded_py_aligned_cues
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
        Ok(SubtitleCues::Cue(aligned_cues?))
    })
    .map_err(|e| AlignmentError::PythonError { error: e })
}

pub fn generate_aligned_metadata(subtitle_document: SubtitleDocument) -> SubtitleMetadata {
    SubtitleMetadata {
        album: subtitle_document.metadata.album,
        title: subtitle_document.metadata.title,
        artists: subtitle_document.metadata.artists,
        languages: subtitle_document.metadata.languages,
        file_path: match subtitle_document.metadata.file_path {
            Some(mut path) => {
                path.set_extension("elrc");
                Some(path)
            }
            None => None,
        },
    }
}
