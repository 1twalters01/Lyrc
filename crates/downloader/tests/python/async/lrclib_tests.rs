use chrono::Duration;
use lyrics::{provider::LyricsDownloader, providers::lrclib::LrclibProvider};
use mpris::track::Track;

#[pyo3_async_runtimes::tokio::test]
async fn test_search() -> pyo3::PyResult<()> {
    let track = Track {
        album: Some(String::from("Lux (Complete Works)")),
        disc_number: Some(1),
        title: String::from("Porcelana"),
        track_number: Some(4),
        duration: Duration::seconds(248),
        id: None,
        genres: Vec::from([String::from("World")]),
        artists: Vec::from([String::from("Rosalía")]),
        album_artists: Vec::from([String::from("Rosalía")]),
        file_path: None,
    };

    let lyrics = LrclibProvider.search(track).await.unwrap();
    println!("{:#?}", lyrics);

    Ok(())
}
