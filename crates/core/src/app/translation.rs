use subtitles::language::Language;
use translation::{
    error::TranslationError,
    messages::{TranslationRequest, TranslationTask},
};

use crate::{app::App, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn start_translation(
        &mut self,
        new_language: Language,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.state.translation_running {
            return Ok(());
        }

        let subtitle_document = match &self.state.subtitle_documents.active() {
            Some(subtitle_document_state) => subtitle_document_state.document.clone(),
            None => return Err(Box::new(TranslationError::NoSubtitles)),
        };

        let task = TranslationTask {
            language: new_language,
            subtitle_document: subtitle_document.clone(),
        };

        self.translation_req_tx
            .send(TranslationRequest::Translate(task))
            .await?;

        self.state.alignment_running = true;

        Ok(())
    }
}
