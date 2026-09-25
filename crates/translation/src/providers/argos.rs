use futures::future::BoxFuture;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use subtitles::{
    language::Language,
    subtitles::{SubtitleCues, SubtitleDocument},
};

use crate::{
    error::TranslationError, helpers::convert_py_cues_to_translated_subtitle_document,
    provider::LyricsTranslator,
};

pub struct ArgosTranslator;

impl LyricsTranslator for ArgosTranslator {
    fn translate(
        &self,
        language: Language,
        subtitle_document: SubtitleDocument,
        locals: TaskLocals,
    ) -> BoxFuture<'_, Result<Option<SubtitleDocument>, TranslationError>> {
        Box::pin(async move {
            let original_language = subtitle_document
                .metadata
                .languages
                .first()
                .ok_or(TranslationError::NoLanguageCode)?;

            let py_translated_cues =
                Self::translate_cues(original_language, &language, &subtitle_document, &locals)
                    .await?;

            convert_py_cues_to_translated_subtitle_document(
                py_translated_cues,
                subtitle_document,
                language,
            )
        })
    }
}

impl ArgosTranslator {
    async fn translate_cues(
        original_language: &Language,
        language: &Language,
        subtitle_document: &SubtitleDocument,
        locals: &TaskLocals,
    ) -> Result<Py<PyAny>, TranslationError> {
        let result = Python::attach(|py| -> PyResult<_> {
            let datetime = py.import("datetime")?;
            let timedelta = datetime.getattr("timedelta")?;

            let service_module = PyModule::import(py, "translator.service")?;
            let cue_module = PyModule::import(py, "translator.models.cue")?;
            let language_module = PyModule::import(py, "translator.models.language")?;
            let provider_module = PyModule::import(py, "translator.providers.argos.provider")?;
            let options_module = PyModule::import(py, "translator.providers.argos.options")?;

            let to_language_py = language_module.getattr("Language")?.call1((
                language.as_name(),
                language.as_native_name(),
                language.as_code_2(),
                language.as_code_3(),
                language.as_flores_200(),
            ))?;

            let original_language_py = language_module.getattr("Language")?.call1((
                original_language.as_name(),
                original_language.as_native_name(),
                original_language.as_code_2(),
                original_language.as_code_3(),
                original_language.as_flores_200(),
            ))?;
            let options = options_module
                .getattr("ArgosOptions")?
                .call1((original_language_py.clone(),))?;

            let argos_translator = provider_module.getattr("ArgosTranslator")?.call0()?;

            argos_translator
                .getattr("install_packages")?
                .call1((to_language_py.clone(), original_language_py))?;

            let providers = PyDict::new(py);
            providers.set_item("argos", argos_translator)?;

            let translation_service = service_module
                .getattr("TranslationService")?
                .call1((providers,))?;

            let lrc_contents = PyList::empty(py);
            match &subtitle_document.cues {
                SubtitleCues::Word(cues) => {
                    for cue in cues {
                        let start =
                            timedelta.call1((0, 0, cue.start.num_microseconds().unwrap_or(0)))?;
                        let end =
                            timedelta.call1((0, 0, cue.end.num_microseconds().unwrap_or(0)))?;
                        let content = cue
                            .words
                            .iter()
                            .map(|word| word.content.clone())
                            .collect::<Vec<String>>()
                            .join("");
                        let py_cue = cue_module.getattr("Cue")?.call1((start, end, content))?;
                        lrc_contents.append(py_cue)?;
                    }
                }
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

            let coroutine = translation_service.call_method1(
                "translate",
                ("argos", lrc_contents, to_language_py, options),
            )?;
            into_future_with_locals(locals, coroutine)
        })?;
        
        result.await
        .map_err(|e| TranslationError::PythonError { error: e })
    }
}
