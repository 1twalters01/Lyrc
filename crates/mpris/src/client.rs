use std::collections::HashMap;

use async_stream::stream;
use chrono::Duration;
use futures_core::Stream;
use futures_util::StreamExt;
use zbus::{
    Connection,
    fdo::PropertiesProxy,
    names::OwnedWellKnownName,
    zvariant::{Dict, OwnedValue, Value},
};

use crate::{
    playback::{PlaybackCommand, PlaybackStatus, PlayerEvent},
    proxy::PlayerProxy,
    track::Track,
};

#[derive(Clone)]
pub struct MprisClient {
    connection: Connection,
    service: OwnedWellKnownName,
    player: String,
}

impl MprisClient {
    pub fn get_service(&self) -> OwnedWellKnownName {
        self.service.clone()
    }

    pub fn get_player(&self) -> String {
        self.player.clone()
    }

    pub async fn connect(player: &str) -> zbus::Result<Self> {
        let connection = Connection::session().await?;
        Self::with_connection(connection, player)
    }

    pub fn with_connection(
        connection: Connection,
        player: &str,
    ) -> zbus::Result<Self> {
        let service = format!("org.mpris.MediaPlayer2.{player}");

        let mpris_client = Self {
            connection,
            service: service.try_into()?,
            player: String::from(player),
        };

        Ok(mpris_client)
    }

    pub async fn find_players() -> zbus::Result<Vec<String>> {
        let connection = Connection::session().await?;
        Self::find_players_on(&connection).await
    }

    async fn find_players_on(
        connection: &Connection,
    ) -> zbus::Result<Vec<String>> {
        let proxy = zbus::fdo::DBusProxy::new(connection).await?;
        let names = proxy.list_names().await?;

        let players = names
            .into_iter()
            .filter_map(|name| {
                name.strip_prefix("org.mpris.MediaPlayer2.")
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();

        Ok(players)
    }

    async fn proxy(&self) -> zbus::Result<PlayerProxy<'_>> {
        PlayerProxy::builder(&self.connection)
            .destination(&self.service)?
            .build()
            .await
    }

    pub async fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>> {
        let proxy = self.proxy().await?;
        let metadata = proxy.metadata().await?;

        Ok(metadata)
    }

    pub async fn get_current_position(&self) -> zbus::Result<Duration> {
        let proxy = self.proxy().await?;
        let micros = proxy.position().await?;
        let current_position = Duration::microseconds(micros as i64);

        Ok(current_position)
    }

    pub async fn get_current_track(&self) -> zbus::Result<Track> {
        let metadata = self.proxy().await?.metadata().await?;
        let current_track = Track::parse_track(self, metadata).await;

        Ok(current_track)
    }

    pub async fn get_playback_status(&self) -> zbus::Result<PlaybackStatus> {
        let proxy = self.proxy().await?;
        let proxy_playback_status = proxy.playback_status().await?;
        let playback_status = PlaybackStatus::parse(&proxy_playback_status);

        Ok(playback_status)
    }

    pub async fn choose_player(
        targets: &[String],
    ) -> zbus::Result<Option<String>> {
        let connection = Connection::session().await?;
        let players = Self::find_players_on(&connection).await?;

        let mut players_with_status = Vec::new();

        for player in &players {
            let Ok(client) = Self::with_connection(connection.clone(), player) else {
                continue;
            };

            let Ok(status) = client.get_playback_status().await else {
                continue;
            };

            players_with_status.push((player, status));
        }

        let player = Self::select_player(&players_with_status, targets);

        Ok(player)
    }

    fn select_player(
        players_with_status: &[(&String, PlaybackStatus)],
        targets: &[String],
    ) -> Option<String> {
        for target in targets {
            if let Some((player, _)) = players_with_status
                .iter()
                .find(|(player, _)| *player == target)
            {
                return Some(player.to_string());
            }
        }

        for status in [
            PlaybackStatus::Playing,
            PlaybackStatus::Paused,
            PlaybackStatus::Stopped,
        ] {
            if let Some((player, _)) = players_with_status
                .iter()
                .find(|(_, player_status)| player_status == &status)
            {
                return Some(player.to_string());
            }
        }

        players_with_status
            .first()
            .map(|(player, _)| player.to_string())
    }

    pub async fn execute(&self, command: PlaybackCommand) -> zbus::Result<()> {
        let proxy = self.proxy().await?;

        match command {
            PlaybackCommand::Play => match self.get_playback_status().await? {
                PlaybackStatus::Playing => {}
                _ => proxy.play().await?,
            },
            PlaybackCommand::Pause => proxy.pause().await?,
            PlaybackCommand::Toggle => proxy.play_pause().await?,
            PlaybackCommand::Next => proxy.next().await?,
            PlaybackCommand::Previous => proxy.previous().await?,
            PlaybackCommand::Seek(offset) => {
                let offset = offset.num_microseconds().ok_or_else(|| {
                    zbus::Error::Failure("Seek offset is out of range".into())
                })?;
                proxy.seek(offset).await?
            }
            PlaybackCommand::SetPosition(position) => {
                let track = self.get_current_track().await?;
                let track_id = track.id.as_ref().ok_or_else(|| {
                    zbus::Error::Failure("Missing track id".into())
                })?;

                let position =
                    position.num_microseconds().ok_or_else(|| {
                        zbus::Error::Failure("Position is out of range".into())
                    })?;

                proxy.set_position(track_id, position).await?
            }
        }

        Ok(())
    }

    pub async fn events(
        &self,
    ) -> zbus::Result<impl Stream<Item = PlayerEvent>> {
        let properties_proxy = PropertiesProxy::builder(&self.connection)
            .destination(&self.service)?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        let player_proxy = PlayerProxy::builder(&self.connection)
            .destination(&self.service)?
            .path("/org/mpris/MediaPlayer2")?
            .build()
            .await?;

        let mut properties_event =
            properties_proxy.receive_properties_changed().await?;
        let mut seeked_event = player_proxy.receive_seeked().await?;

        let output = stream! {
            loop {
                tokio::select! {
                    Some(signal) = properties_event.next() => {
                        let Ok(args) = signal.args() else {
                            continue;
                        };

                        let changed = args.changed_properties();

                        if let Some(Value::Dict(metadata)) = changed.get("Metadata") {
                            let owned_metadata = Self::own_metadata(metadata);
                            let track = Track::parse_track(self, owned_metadata).await;
                            yield PlayerEvent::TrackChanged(track);
                        }

                        if let Some(Value::Str(status)) = changed.get("PlaybackStatus") {
                            let playback_status = PlaybackStatus::parse(status);
                            yield PlayerEvent::PlaybackChanged(playback_status);
                        }
                    }

                    Some(signal) = seeked_event.next() => {
                        let Ok(args) = signal.args() else {
                            continue;
                        };

                        let position = Duration::microseconds(*args.position());
                        yield PlayerEvent::Seeked(position);
                    }

                    else => break,
                }
            }
        };

        Ok(Box::pin(output))
    }

    fn own_metadata(metadata: &Dict<'_, '_>) -> HashMap<String, OwnedValue> {
        metadata
            .iter()
            .filter_map(|(k, v)| {
                let key = match k {
                    Value::Str(s) => s.to_string(),
                    _ => return None,
                };
                let value = OwnedValue::try_from(v.clone()).ok()?;
                Some((key, value))
            })
            .collect()
    }
}
