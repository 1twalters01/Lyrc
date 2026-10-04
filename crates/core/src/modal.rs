use downloader::service::DownloadService;
use subtitles::subtitles::SyncLevel;

use crate::state::SubtitleVariant;

#[derive(Debug, Clone, PartialEq)]
pub enum ModalOption {
    Player,
    Download,
    Translate,
    Alignment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalError {
    InvalidVariant,
    TranslationFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    Selection {
        options: Vec<ModalOption>,
        new_modal: ModalOption,
    },
    Player {
        players: Vec<String>,
        new_player: Option<String>,
        error: Option<ModalError>,
    },
    Translate {
        input: String,
        input_variant: Option<SubtitleVariant>,
        new_variant: Option<SubtitleVariant>,
        error: Option<ModalError>,
    },
    Alignment {
        new_alignment: SyncLevel,
        error: Option<ModalError>,
    },
    Download {
        providers: Vec<DownloadService>,
        new_provider: DownloadService,
        error: Option<ModalError>,
    },
}
