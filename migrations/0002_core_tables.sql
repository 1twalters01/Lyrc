CREATE TABLE artist (
    uuid BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE song (
    uuid BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL
);

CREATE TABLE song_artist (
    song_uuid   BLOB NOT NULL,
    artist_uuid BLOB NOT NULL,
    PRIMARY KEY (song_uuid, artist_uuid),
    FOREIGN KEY (song_uuid)
        REFERENCES song(uuid)
        ON DELETE CASCADE,
    FOREIGN KEY (artist_uuid)
        REFERENCES artist(uuid)
        ON DELETE CASCADE
);

CREATE TABLE recording (
    uuid BLOB PRIMARY KEY,
    song_uuid BLOB NOT NULL,
    duration_ms INTEGER NOT NULL,
    FOREIGN KEY(song_uuid)
        REFERENCES song(uuid)
        ON DELETE CASCADE
);

CREATE TABLE recording_artist (
    recording_uuid BLOB NOT NULL,
    artist_uuid    BLOB NOT NULL,
    PRIMARY KEY (recording_uuid, artist_uuid),
    FOREIGN KEY (recording_uuid)
        REFERENCES recording(uuid)
        ON DELETE CASCADE,
    FOREIGN KEY (artist_uuid)
        REFERENCES artist(uuid)
        ON DELETE CASCADE
);

CREATE TABLE recording_identifier (
    uuid BLOB PRIMARY KEY,
    recording_uuid BLOB NOT NULL,
    source_id INTEGER NOT NULL,
    identifier TEXT NOT NULL,
    FOREIGN KEY(recording_uuid)
        REFERENCES recording(uuid)
        ON DELETE CASCADE,
    FOREIGN KEY(source_id)
        REFERENCES source(id)
        ON DELETE CASCADE
    UNIQUE(source_id, identifier)
);

CREATE INDEX recording_song_idx ON recording(song_uuid);
CREATE INDEX recording_identifier_recording_idx
    ON recording_identifier(recording_uuid);
