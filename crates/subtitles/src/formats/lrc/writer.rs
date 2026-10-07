use chrono::Duration;

use crate::{
    formats::lrc::error::LrcError,
    subtitles::{SubtitleCues, SubtitleDocument, SubtitleMetadata},
    writer::SubtitleWriter,
};

pub struct LrcWriter;

impl SubtitleWriter for LrcWriter {
    type Error = LrcError;

    fn write(&self, subtitle_document: &SubtitleDocument) -> Result<String, Self::Error> {
        let mut file = String::new();

        Self::write_metadata(&subtitle_document.metadata, &mut file);
        file.push_str("\n");
        Self::write_cues(subtitle_document, &mut file)?;
        // file.trim();

        Ok(file)
    }
}

impl LrcWriter {
    fn write_metadata(subtitle_metadata: &SubtitleMetadata, file: &mut String) {
        if let Some(title) = &subtitle_metadata.title {
            file.push_str(&format!("[ti:{title}]\n"));
        }

        if let Some(album) = &subtitle_metadata.album {
            file.push_str(&format!("[al:{album}]\n"));
        }

        if !subtitle_metadata.artists.is_empty() {
            file.push_str(&format!("[ar:{}]\n", subtitle_metadata.artists.join(", ")));
        }

        if !subtitle_metadata.languages.is_empty() {
            file.push_str(&format!(
                "[la:{}]\n",
                subtitle_metadata
                    .languages
                    .iter()
                    .map(|l| l.as_name())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ));
        }
    }

    fn write_cues(document: &SubtitleDocument, output: &mut String) -> Result<(), LrcError> {
        let lines = match &document.cues {
            SubtitleCues::Word(cues) => cues
                .iter()
                .map(|cue| {
                    let start_time = Self::format_timestamp(cue.start);
                    let text = cue
                        .words
                        .iter()
                        .map(|word| word.content.clone())
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("[{}]{}\n", start_time, text)
                })
                .collect::<Vec<String>>()
                .join("\n"),
            SubtitleCues::Cue(cues) => cues
                .iter()
                .map(|cue| {
                    let start_time = Self::format_timestamp(cue.start);
                    let text = cue.content.clone();
                    format!("[{}]{}", start_time, text)
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => return Err(LrcError::InvalidSubtitleDocumentFormat),
        };

        output.push_str(&lines);
        Ok(())
    }

    fn format_timestamp(duration: Duration) -> String {
        let minutes = duration.num_minutes();
        let seconds = duration.num_seconds() % 60;
        let milliseconds = duration.num_milliseconds() % 1_000;

        format!("{minutes:02}:{seconds:02}.{milliseconds:03}")
    }
}
