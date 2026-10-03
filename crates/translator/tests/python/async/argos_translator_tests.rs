use std::{path::PathBuf, str::FromStr};

use subtitles::{language::Language, subtitles::SubtitleDocument};
use translation::{provider::LyricsTranslator, providers::argos::ArgosTranslator};

#[pyo3_async_runtimes::tokio::test]
async fn translator_test() -> pyo3::PyResult<()> {
    let root_file_str = String::from(
        "/data/Languages/Spanish/music/Kali Uchis/2020 - Sin Miedo (del Amor y Otros Demonios) ∞ (Deluxe Vers)/09 de nadie",
    );
    // let audio_file_string = root_file_str.clone() + ".flac";
    let lrc_file_string = root_file_str + ".lrc";

    let lrc_file_path = PathBuf::from_str(&lrc_file_string).unwrap();
    let new_language = Language::French;

    let subtitle_document = SubtitleDocument::from_pathbuf(lrc_file_path).unwrap();
    let translated_subtitle_document = ArgosTranslator
        .translate(new_language, subtitle_document)
        .await
        .unwrap();
    println!("{:#?}", translated_subtitle_document);

    Ok(())
}
