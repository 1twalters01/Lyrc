CREATE TABLE lyrics_variant (
    id INTEGER PRIMARY KEY,
    translation_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    FOREIGN KEY(translation_id) REFERENCES translation(id),
    FOREIGN KEY(language_id) REFERENCES language(id),
    UNIQUE(translation_id, language_id)
);

CREATE TABLE lyrics_file (
    uuid BLOB PRIMARY KEY,
    recording_uuid BLOB NOT NULL,
    variant_id INTEGER NOT NULL,
    format_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    FOREIGN KEY(recording_uuid) REFERENCES recording(uuid),
    FOREIGN KEY(variant_id) REFERENCES lyrics_variant(id),
    FOREIGN KEY(format_id) REFERENCES lyrics_format(id),
    UNIQUE(recording_uuid, variant_id, format_id)
);

CREATE TABLE audio_file (
    uuid BLOB PRIMARY KEY,
    recording_uuid BLOB NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    FOREIGN KEY(recording_uuid) REFERENCES recording(uuid)
);

CREATE INDEX audio_file_recording_idx
    ON audio_file(recording_uuid);
