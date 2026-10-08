use crate::{app::App, renderer::Renderer};

impl<R> App<R>
where
    R: Renderer,
{
    pub async fn update_track(&mut self) {
        self.state
            .update_track(self.mpris_client.get_current_track().await.ok())
            .await;
    }

    pub async fn update_subtitle_document(&mut self) {
        // Get audio file from db using audio file path
        // Get the recording_uuid
        // Get lyrics file from recording_uuid
        // If not possible then:
        self.state.reload_subtitle_documents().await;
    }
}
