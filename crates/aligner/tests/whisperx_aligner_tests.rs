use std::{path::PathBuf, str::FromStr};

use alignment::{provider::LyricsAligner, providers::whisperx::WhisperXAligner};
use subtitles::subtitles::SubtitleDocument;

#[test]
fn aligner_test() {
    let root_file_str = String::from(
        "/data/Languages/Spanish/music/Kali Uchis/2020 - Sin Miedo (del Amor y Otros Demonios) ∞ (Deluxe Vers)/09 de nadie",
    );
    let audio_file_string = root_file_str.clone() + ".flac";
    let lrc_file_string = root_file_str + ".lrc";

    let audio_file_path = PathBuf::from_str(&audio_file_string).unwrap();
    let lrc_file_path = PathBuf::from_str(&lrc_file_string).unwrap();

    let subtitle_document = SubtitleDocument::from_pathbuf(lrc_file_path).unwrap();
    let aligned_subtitle_document = WhisperXAligner::align_cues(audio_file_path, subtitle_document);
    println!("{:#?}", aligned_subtitle_document);
    assert!(false)
}
