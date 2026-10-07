use sqlx::{Pool, Sqlite};
use subtitles::subtitles::SubtitleDocument;

use crate::models::{artist::ArtistRow, song::SongRow, song_artist::SongArtistRow};

pub struct SubtitleDocumentRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> SubtitleDocumentRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn save(&self, document: &SubtitleDocument) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let artist_names = &document.metadata.artists;
        let song_title = match &document.metadata.title {
            Some(song_title) => song_title,
            None => return Err(sqlx::Error::InvalidArgument(String::from("No song title"))),
        };

        let artist_rows = ArtistRow::get_or_create_by_names(&mut tx, artist_names).await?;
        let song_rows =
            if SongArtistRow::exists_by_name(&mut tx, song_title, &artist_names[0]).await? {
                SongRow::get_or_create_by_titles(&mut tx, &[song_title.to_string()]).await?
            } else {
                let song_row = SongRow::from_title(song_title);
                song_row.insert(&mut tx).await?;
                Vec::from([song_row])
            };
        println!("song rows: {:?}", song_rows);

        tx.commit().await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<SubtitleDocument, sqlx::Error> {
        todo!()
    }

    pub async fn delete(&self, document: &SubtitleDocument) -> Result<bool, sqlx::Error> {
        todo!()
    }
}
