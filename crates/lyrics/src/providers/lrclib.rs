use futures::future::BoxFuture;
use mpris::track::Track;
use pyo3::{prelude::*, types::PyDict};
use pyo3_async_runtimes::tokio::into_future;

use crate::{
    error::LyricsError,
    models::{Lyrics, LyricsFormat, LyricsSource},
    provider::LyricsDownloader,
};

pub struct LrclibProvider;

impl LyricsDownloader for LrclibProvider {
    fn search(&self, track: Track) -> BoxFuture<'static, Result<Option<Lyrics>, LyricsError>> {
        Box::pin(async move {
            let py_future = Python::attach(|py| -> PyResult<_> {
                // import required python packages
                let httpx = PyModule::import(py, "httpx")?;
                let datetime = PyModule::import(py, "datetime")?;

                // import modules
                let service_module = PyModule::import(py, "downloader.service")?;
                let track_module = PyModule::import(py, "downloader.models.track")?;
                let provider_module = PyModule::import(py, "downloader.providers.lrclib.provider")?;
                let options_module = PyModule::import(py, "downloader.providers.lrclib.options")?;

                // get provider dict with lrclib instance inside
                let client = httpx.getattr("AsyncClient")?.call0()?;
                let lrclib_downloader = provider_module
                    .getattr("LrcLibDownloader")?
                    .call1((client,))?;
                let providers = PyDict::new(py);
                providers.set_item("lrclib", lrclib_downloader)?;

                // Create options
                let options = options_module.getattr("LrcLibOptions")?.call0()?;

                // Create instance of lyrics service
                let lyrics_service = service_module
                    .getattr("LyricsService")?
                    .call1((providers,))?;
                // println!("lyrics service: {:#?}", lyrics_service.is_none());

                // get python track class
                let timedelta = datetime.getattr("timedelta")?.call1((
                    0,
                    0,
                    track.duration.num_microseconds().unwrap_or(0),
                ))?;
                let py_track = track_module.getattr("Track")?.call1((
                    track.title,
                    track.artists.first(),
                    track.album,
                    timedelta,
                ))?;

                let coroutine =
                    lyrics_service.call_method1("search", (py_track, "lrclib", options))?;
                into_future(coroutine)
            })
            .map_err(|e| LyricsError::PythonError { error: e })?;

            let result = py_future
                .await
                .map_err(|e| LyricsError::PythonError { error: e })?;

            let lyrics: Option<Lyrics> = Python::attach(|py| {
                if result.is_none(py) {
                    return Ok(None);
                }

                let lyrics = result.bind(py);
                let content: String = lyrics.getattr("content")?.extract()?;
                let py_format: String = lyrics.getattr("format")?.getattr("value")?.extract()?;
                let py_source: String = lyrics.getattr("source")?.getattr("value")?.extract()?;

                let format = match py_format.as_str() {
                    "lrc" => LyricsFormat::Lrc,
                    "text" => LyricsFormat::Text,
                    _ => {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "unknown lyrics format",
                        ));
                    }
                };
                let source = match py_source.as_str() {
                    "self" => LyricsSource::Python,
                    "lrclib" => LyricsSource::Lrclib,
                    _ => {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "unknown lyrics source",
                        ));
                    }
                };

                Ok(Some(Lyrics {
                    content,
                    format,
                    source,
                }))
            })
            .map_err(|e| LyricsError::PythonError { error: e })?;

            Ok(lyrics)
        })
    }
}
