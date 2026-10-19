use std::{sync::Arc, time::Duration};

use async_channel::Sender;
use configuration::config::Config;
use mpris::client::MprisClient;
use tokio::time::interval;

use crate::interfaces::gui::message::AppMessage;

pub fn event_source(
    config: Config,
    mpris_client: Arc<Option<MprisClient>>,
    tx_mpris: Sender<AppMessage>,
) {
    tokio::spawn(async move {
        let mut tick_event =
            interval(Duration::from_secs_f64(1f64 / config.fps));

        loop {
            tokio::select! {
                _ = tick_event.tick() => {
                    if let Some(ref client) = *mpris_client {
                        let fetch_result = tokio::time::timeout(
                            Duration::from_millis(200),
                            async {
                                let position = client.get_current_position().await;
                                let status = client.get_playback_status().await;
                                (position, status)
                            }
                        ).await;

                        match fetch_result {
                            Ok((Ok(pos), Ok(status))) => {
                                let _ = tx_mpris.send(AppMessage::TickEvent {
                                    new_position: Some(pos),
                                    playback_status: status,
                                }).await;
                            }
                            Ok((e1, e2)) => {
                                eprintln!("BG: MPRIS fetch error - pos: {:?}, status: {:?}", e1, e2);
                            }
                            Err(_) => {
                                eprintln!("BG: TIMED OUT fetching MPRIS position!");
                            }
                        }
                    } else {
                        eprintln!("BG: WARNING: mpris_client_bg is None!");
                    }
                }
            }
        }
    });
}
