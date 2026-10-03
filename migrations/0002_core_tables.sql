CREATE TABLE artist (
    uuid BLOB PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE song (
    uuid BLOB PRIMARY KEY,
    title TEXT NOT NULL,
    artist_uuid BLOB NOT NULL,
    FOREIGN KEY(artist_uuid) REFERENCES artist(uuid)
);

CREATE TABLE recording (
    uuid BLOB PRIMARY KEY,
    song_uuid BLOB NOT NULL,
    artist_uuid BLOB NOT NULL,
    duration_ms INTEGER NOT NULL,
    FOREIGN KEY(song_uuid) REFERENCES song(uuid),
    FOREIGN KEY(artist_uuid) REFERENCES artist(uuid)
);

CREATE TABLE recording_identifier (
    uuid BLOB PRIMARY KEY,
    recording_uuid BLOB NOT NULL,
    source_id INTEGER NOT NULL,
    identifier TEXT NOT NULL,
    FOREIGN KEY(recording_uuid) REFERENCES recording(uuid),
    FOREIGN KEY(source_id) REFERENCES source(id),
    UNIQUE(source_id, identifier)
);

CREATE INDEX recording_song_idx ON recording(song_uuid);
CREATE INDEX recording_artist_idx ON recording(artist_uuid);
CREATE INDEX recording_identifier_recording_idx
    ON recording_identifier(recording_uuid);
