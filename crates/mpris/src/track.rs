use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::process::Command;

use chrono::Duration;
use url::Url;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

use lofty::{file::TaggedFileExt, read_from_path, tag::ItemKey};

use crate::client::MprisClient;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub album: Option<String>,
    pub disc_number: Option<i32>,
    pub title: String,
    pub track_number: Option<i32>,
    pub duration: Duration,
    pub id: Option<OwnedObjectPath>,
    pub genres: Vec<String>,
    pub artists: Vec<String>,
    pub album_artists: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub isrc: Option<String>,
}

impl Track {
    pub async fn parse_track(mpris: &MprisClient, metadata: HashMap<String, OwnedValue>) -> Track {
        let album = get_optional_string(&metadata, "xesam:album");
        let disc_number = get_optional_i32(&metadata, "xesam:discNumber");
        let title = get_string(&metadata, "xesam:title");
        let track_number = get_optional_i32(&metadata, "xesam:trackNumber");
        let duration = get_duration(&metadata, "mpris:length");
        let id = get_object_path(&metadata, "mpris:trackid");
        let genres = get_string_array(&metadata, "xesam:genre");
        let artists = get_string_array(&metadata, "xesam:artist");
        let album_artists = get_string_array(&metadata, "xesam:albumArtist");
        let file_path = if mpris.get_player() == "cmus" {
            get_current_track_file_path_cmus().await.ok().flatten()
        } else {
            get_optional_pathbuf(&metadata, "xesam:url")
        };
        let isrc = file_path
            .as_ref()
            .and_then(|file_path| get_isrc(&file_path).unwrap_or(None));
        // println!("\nisrc: {:?}", isrc);

        Track {
            album,
            disc_number,
            title,
            track_number,
            duration,
            id,
            genres,
            artists,
            album_artists,

            file_path,
            isrc,
        }
    }

    pub fn get_elrc_file_path(&self) -> Option<PathBuf> {
        if let Some(track_file_path) = self.file_path.clone() {
            let mut lyrics_file_path = track_file_path.to_path_buf();
            lyrics_file_path.set_extension("elrc");
            return Some(lyrics_file_path);
        }
        None
    }

    pub fn get_lrc_file_path(&self) -> Option<PathBuf> {
        if let Some(track_file_path) = self.file_path.clone() {
            let mut lyrics_file_path = track_file_path.to_path_buf();
            lyrics_file_path.set_extension("lrc");
            return Some(lyrics_file_path);
        }
        None
    }

    pub fn get_txt_file_path(&self) -> Option<PathBuf> {
        if let Some(track_file_path) = self.file_path.clone() {
            let mut lyrics_file_path = track_file_path.to_path_buf();
            lyrics_file_path.set_extension("txt");
            return Some(lyrics_file_path);
        }
        None
    }
}

fn get_string(metadata: &HashMap<String, OwnedValue>, key: &str) -> String {
    metadata
        .get(key)
        .and_then(|v| v.downcast_ref::<String>().ok())
        .unwrap_or_default()
}

fn get_optional_string(metadata: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|v| v.downcast_ref::<String>().ok())
}

fn get_optional_pathbuf(metadata: &HashMap<String, OwnedValue>, key: &str) -> Option<PathBuf> {
    metadata
        .get(key)
        .and_then(|v| v.downcast_ref::<String>().ok())
        .and_then(|s| Url::parse(&s).ok())
        .and_then(|url| url.to_file_path().ok())
}

fn get_optional_i32(metadata: &HashMap<String, OwnedValue>, key: &str) -> Option<i32> {
    metadata.get(key).and_then(|v| v.downcast_ref::<i32>().ok())
}

fn get_string_array(metadata: &HashMap<String, OwnedValue>, key: &str) -> Vec<String> {
    let Some(value) = metadata.get(key) else {
        return Vec::new();
    };

    let value: &Value = value;

    match value {
        Value::Value(boxed) => match boxed.as_ref() {
            Value::Array(array) => array
                .iter()
                .filter_map(|v| match v {
                    Value::Str(s) => Some(s.to_string()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        },
        Value::Array(array) => array
            .iter()
            .filter_map(|v| match v {
                Value::Str(s) => Some(s.to_string()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn get_duration(metadata: &HashMap<String, OwnedValue>, key: &str) -> Duration {
    metadata
        .get(key)
        .and_then(|v| v.downcast_ref::<i64>().ok())
        .map(|v| Duration::microseconds(v))
        .unwrap_or_default()
}

fn get_object_path(metadata: &HashMap<String, OwnedValue>, key: &str) -> Option<OwnedObjectPath> {
    metadata
        .get(key)
        .and_then(|v| v.downcast_ref::<ObjectPath>().ok())
        .map(OwnedObjectPath::from)
}

async fn get_current_track_file_path_cmus() -> Result<Option<PathBuf>, std::io::Error> {
    let output = Command::new("cmus-remote").arg("-Q").output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("file ") {
            return Ok(Some(PathBuf::from(path)));
        }
    }

    Ok(None)
}

fn get_isrc(path: &Path) -> Result<Option<String>, lofty::error::FileParseError> {
    let tagged_file = read_from_path(path)?;

    let isrc = tagged_file
        .tags()
        .iter()
        .find_map(|tag| tag.get_string(ItemKey::Isrc))
        .map(str::to_owned);

    Ok(isrc)
}
