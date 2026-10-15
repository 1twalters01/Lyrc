use crate::{
    formats::txt::error::TxtError,
    subtitles::{SubtitleCues, SubtitleDocument, SubtitleMetadata},
    writer::SubtitleWriter,
};

pub struct TxtWriter;

impl SubtitleWriter for TxtWriter {
    type Error = TxtError;

    fn write(
        &self,
        subtitle_document: &SubtitleDocument,
    ) -> Result<String, Self::Error> {
        let mut file = String::new();

        Self::write_metadata(&subtitle_document.metadata, &mut file);
        Self::write_cues(subtitle_document, &mut file)?;
        // file.trim();

        Ok(file)
    }
}

impl TxtWriter {
    fn write_metadata(subtitle_metadata: &SubtitleMetadata, file: &mut String) {
        if let Some(title) = &subtitle_metadata.title {
            file.push_str(&format!("[ti:{title}]\n"));
        }

        if let Some(album) = &subtitle_metadata.album {
            file.push_str(&format!("[al:{album}]\n"));
        }

        if !subtitle_metadata.artists.is_empty() {
            file.push_str(&format!(
                "[ar:{}]\n",
                subtitle_metadata.artists.join(", ")
            ));
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

    fn write_cues(
        document: &SubtitleDocument,
        output: &mut String,
    ) -> Result<(), TxtError> {
        let lines = match &document.cues {
            SubtitleCues::Word(cues) => cues
                .iter()
                .map(|cue| {
                    let text = cue
                        .words
                        .iter()
                        .map(|word| word.content.clone())
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("{}\n", text)
                })
                .collect::<Vec<String>>()
                .join("\n"),
            SubtitleCues::Cue(cues) => cues
                .iter()
                .map(|cue| {
                    let text = cue.content.clone();
                    format!("{}\n", text)
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => return Err(TxtError::InvalidSubtitleDocumentFormat),
        };

        output.push_str(&lines);
        Ok(())
    }
}
