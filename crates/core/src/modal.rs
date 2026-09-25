use subtitles::subtitles::SyncLevel;

use crate::state::SubtitleVariant;

#[derive(Clone, PartialEq)]
pub enum ModalError {
    InvalidVariant,
    TranslationFailed,
}

#[derive(Clone, PartialEq)]
pub enum Modal {
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
}
