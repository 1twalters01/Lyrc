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
        self.state.reload_subtitle_documents().await;
    }
}
