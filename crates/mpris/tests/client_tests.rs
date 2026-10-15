use chrono::Duration;
use futures_util::stream::StreamExt;

use mpris::{
    client::MprisClient,
    playback::{PlaybackCommand, PlaybackStatus},
};

#[tokio::test]
async fn connect_to_cmus() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    assert_eq!(client.get_service().as_str(), "org.mpris.MediaPlayer2.cmus");
}

#[tokio::test]
async fn get_current_position() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let current_position = client.get_current_position().await.unwrap();
    assert_ne!(current_position, chrono::Duration::microseconds(5));
}

#[tokio::test]
#[ignore = "takes 2 seconds"]
async fn poll_current_position() {
    let client = MprisClient::connect("cmus")
        .await
        .expect("failed to connect to cmus");

    for _ in 0..20 {
        let position = client
            .get_current_position()
            .await
            .expect("failed to get position");

        println!(
            "Position: {}s {}µs",
            position.num_seconds(),
            position.num_microseconds().unwrap()
        );

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn get_current_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let current_track = client.get_current_track().await.unwrap();

    assert_ne!(current_track.title, String::new());
    assert!(
        current_track
            .file_path
            .and_then(|p| Some(p.exists()))
            .is_some()
    );
}

#[tokio::test]
async fn get_playback_status() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let playback_status = client.get_playback_status().await;

    assert!(playback_status.is_ok());
}

#[tokio::test]
#[ignore = "changes playback status"]
async fn execute_play_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    client.execute(PlaybackCommand::Play).await.unwrap();
    let playback_status = client.get_playback_status().await.unwrap();

    assert_eq!(playback_status, PlaybackStatus::Playing);
}

#[tokio::test]
#[ignore = "changes playback status"]
async fn execute_pause_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    client.execute(PlaybackCommand::Pause).await.unwrap();
    let playback_status = client.get_playback_status().await.unwrap();

    assert_eq!(playback_status, PlaybackStatus::Paused);
}

#[tokio::test]
#[ignore = "changes playback status"]
async fn execute_toggle_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let playback_status = client.get_playback_status().await.unwrap();
    client.execute(PlaybackCommand::Toggle).await.unwrap();
    let new_playback_status = client.get_playback_status().await.unwrap();

    if playback_status == PlaybackStatus::Playing {
        assert_eq!(new_playback_status, PlaybackStatus::Paused);
    } else if playback_status == PlaybackStatus::Paused {
        assert_eq!(new_playback_status, PlaybackStatus::Playing);
    }
}

#[tokio::test]
#[ignore = "changes current track"]
async fn execute_next_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let current_track = client.get_current_track().await.unwrap();
    client.execute(PlaybackCommand::Next).await.unwrap();
    let next_track = client.get_current_track().await.unwrap();

    assert_eq!(
        current_track.track_number.and_then(|n| Some(n + 1)),
        next_track.track_number
    );
}

#[tokio::test]
#[ignore = "changes current track"]
async fn execute_previous_track() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let current_track = client.get_current_track().await.unwrap();
    client.execute(PlaybackCommand::Previous).await.unwrap();
    let next_track = client.get_current_track().await.unwrap();

    assert_eq!(
        current_track.track_number.and_then(|n| Some(n - 1)),
        next_track.track_number
    );
}

#[tokio::test]
#[ignore = "changes current track position"]
async fn execute_seek() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let target_position =
        Duration::seconds(10) + Duration::microseconds(251000);

    let current_position = client.get_current_position().await.unwrap();

    client
        .execute(PlaybackCommand::Seek(target_position))
        .await
        .unwrap();

    let new_position = client.get_current_position().await.unwrap();
    assert_eq!(current_position + target_position, new_position);
}

#[tokio::test]
#[ignore = "changes current track position"]
async fn execute_set_position() {
    let player = "cmus";
    let client = MprisClient::connect(player)
        .await
        .expect("failed to connect to cmus");

    let target_position =
        Duration::seconds(10) + Duration::microseconds(251000);

    let current_position = client.get_current_position().await.unwrap();
    assert_ne!(current_position, target_position);

    client
        .execute(PlaybackCommand::SetPosition(target_position))
        .await
        .unwrap();

    let current_position = client.get_current_position().await.unwrap();
    assert_eq!(current_position, target_position);
}

#[tokio::test]
// #[ignore = "relies on playback changing"]
async fn events_test() {
    let client = MprisClient::connect("cmus").await.unwrap();

    let mut events = client.events().await.unwrap();

    let mut count = 0;
    let event_limit = 3;
    while let Some(event) = events.next().await {
        println!("count: {:#?}", count);
        println!("{event:?}");
        count += 1;
        if count == event_limit {
            break;
        }
    }
    assert!(false)
}
