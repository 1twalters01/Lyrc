use std::path::PathBuf;

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use subtitles::{
    language::Language,
    subtitles::{SubtitleCues, SubtitleDocument},
};

use crate::{
    error::AlignmentError, helpers::convert_py_cues_to_word_aligned_subtitle_document,
    provider::LyricsAligner,
};

pub struct WhisperXWordAligner;

impl LyricsAligner for WhisperXWordAligner {
    fn align_cues(
        audio_file_path: PathBuf,
        subtitle_document: SubtitleDocument,
    ) -> Result<Option<SubtitleDocument>, AlignmentError> {
        let audio_path = audio_file_path
            .to_str()
            .ok_or(AlignmentError::InvalidAudioPath)?
            .to_owned();

        let language = subtitle_document
            .metadata
            .languages
            .first()
            .ok_or(AlignmentError::NoLanguageCode)?;

        let device = "cuda"; // Store in Config crate

        let py_aligned_cues = Self::align_cues(&subtitle_document, audio_path, language, device)?;

        convert_py_cues_to_word_aligned_subtitle_document(py_aligned_cues, subtitle_document)
    }
}

impl WhisperXWordAligner {
    fn align_cues(
        subtitle_document: &SubtitleDocument,
        audio_path: String,
        language: &Language,
        device: &str,
    ) -> Result<Py<PyAny>, AlignmentError> {
        Python::attach(|py| -> Result<Py<PyAny>, AlignmentError> {
            let datetime = py.import("datetime")?;
            let timedelta = datetime.getattr("timedelta")?;

            let service_module = PyModule::import(py, "aligner.service")?;
            let language_module = PyModule::import(py, "aligner.models.language")?;
            let cue_module = PyModule::import(py, "aligner.models.cue")?;
            let provider_module = PyModule::import(py, "aligner.whisperx_word.provider")?;
            let options_module = PyModule::import(py, "aligner.whisperx_word.options")?;

            let whisperx_aligner = provider_module
                .getattr("WhisperXWordAligner")?
                .call1((device,))?;
            let providers = PyDict::new(py);
            providers.set_item("whisperx_word", whisperx_aligner)?;

            let alignment_service = service_module
                .getattr("AlignmentService")?
                .call1((providers,))?;

            let language_py = language_module.getattr("Language")?.call1((
                language.as_name(),
                language.as_native_name(),
                language.as_code_2(),
                language.as_code_3(),
                language.as_flores_200(),
            ))?;
            let options = options_module
                .getattr("WhisperXWordOptions")?
                .call1((language_py,))?;

            let lrc_contents = PyList::empty(py);
            match &subtitle_document.cues {
                SubtitleCues::Word(_) => return Err(AlignmentError::AlreadyAligned),
                SubtitleCues::Cue(cues) => {
                    for cue in cues {
                        let start =
                            timedelta.call1((0, 0, cue.start.num_microseconds().unwrap_or(0)))?;
                        let end =
                            timedelta.call1((0, 0, cue.end.num_microseconds().unwrap_or(0)))?;
                        let content = cue.content.clone();
                        let py_cue = cue_module.getattr("Cue")?.call1((start, end, content))?;
                        lrc_contents.append(py_cue)?;
                    }
                }
                SubtitleCues::Line(lines) => {
                    for line in lines {
                        let content = line.content.clone();
                        lrc_contents.append(content)?;
                    }
                }
                // SubtitleCues::None => return Err(TranslationError::NoSubtitles),
                SubtitleCues::None => {}
            }

            let result = alignment_service.call_method1(
                "align_cues",
                ("whisperx_word", lrc_contents, audio_path, options),
            )?;

            Ok(result.unbind())
        })
    }
}
