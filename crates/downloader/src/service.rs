use configuration::config::Config;

use crate::{
    provider::LyricsDownloader,
    providers::{lrclib::LrclibProvider, musixmatch::MusixmatchProvider},
};

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadService {
    LibLrc,
    Musixmatch,
}

impl DownloadService {
    pub fn as_str(&self) -> String {
        match self {
            Self::LibLrc => String::from("LibLrc"),
            Self::Musixmatch => String::from("Musixmatch"),
        }
    }

    pub fn get_provider(&self) -> Box<dyn LyricsDownloader> {
        match self {
            Self::LibLrc => Box::new(LrclibProvider) as Box<dyn LyricsDownloader>,
            Self::Musixmatch => Box::new(MusixmatchProvider) as Box<dyn LyricsDownloader>,
        }
    }

    pub fn get_prefered_provider(config: &Config) -> Self {
        Self::LibLrc
    }

    pub fn get_providers(config: &Config) -> Vec<DownloadService> {
        return Vec::from([Self::LibLrc, Self::Musixmatch]);
    }
}
