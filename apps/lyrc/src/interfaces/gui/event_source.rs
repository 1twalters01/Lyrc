use std::{sync::Arc, time::Duration};

use async_channel::Sender;
use configuration::config::Config;
use futures_util::stream::StreamExt;
use mpris::client::MprisClient;
use tokio::time::interval;

use crate::{interfaces::gui::message::AppMessage, workers::start::WorkersRx};

pub fn event_source(
    config: Config,
    mpris_client: Arc<Option<MprisClient>>,
    mut workers_rx: WorkersRx,
    tx_mpris: Sender<AppMessage>,
) {
    tokio::spawn(async move {
        let mut mpris_event = match mpris_client.as_ref() {
            Some(mpris_client) => match mpris_client.events().await {
                Ok(mpris_event) => Some(mpris_event),
                Err(err) => {
                    eprintln!("Mpris events stream failed: {:?}", err);
                    None
                }
            },
            None => None,
        };

        let mut tick_event =
            interval(Duration::from_secs_f64(1f64 / config.fps));

        if let Some(ref client) = *mpris_client {
            let track = client.get_current_track().await.ok();
            let _ =
                tx_mpris.send(AppMessage::FirstEvent { track: track }).await;
        }

        loop {
            tokio::select! {
                Some(event) = async {
                    match mpris_event.as_mut() {
                        Some(events) => events.next().await,
                        None => std::future::pending().await,
                    }
                } => {
                    let _ = tx_mpris.send(AppMessage::PlayerEvent { event }).await;
                },

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
                            Ok((pos_err, status_err)) => {
                                eprintln!("Mpris fetch error - pos: {:?}, status: {:?}", pos_err, status_err);
                            }
                            Err(_) => {
                                eprintln!("Mpris position fetch timed out");
                            }
                        }
                    } else {
                        eprintln!("No mpris client");
                    }
                }

                Some(result) = workers_rx.download_rx.recv() => {
                    let _ = tx_mpris.send(AppMessage::DownloadEvent { result }).await;
                },

                Some(result) = workers_rx.alignment_rx.recv() => {
                    let _ = tx_mpris.send(AppMessage::AlignmentEvent { result }).await;
                },

                Some(result) = workers_rx.translation_rx.recv() => {
                    let _ = tx_mpris.send(AppMessage::TranslationEvent { result }).await;
                },
            }
        }
    });
}
