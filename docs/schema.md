# Schema
Field               Type                Description

## Artist -> migrations/0002_core_tables.sql
uuid                UUID
name                TEXT

## Song -> migrations/0002_core_tables.sql 
uuid                UUID
title               TEXT
artist_uuid         UUID

## Recording -> migrations/0002_core_tables.sql 
uuid                UUID
song_uuid           Index(UUID)
artist_uuid         Index(UUID)         Could be a cover so artist needed here as well
duration_ms         INTEGER

## Source -> migrations/0001_fixed_tables.sql
id                  INTEGER
source              TEXT                e.g. musicbrainz, or liblrc

## Recording Identifier -> migrations/0002_core_tables.sql
uuid                UUID
recording_uuid      Index(UUID)
source_id           INTEGER             Source of the recording identifier
identifier          TEXT                Identifier from the source
UNIQUE(source_id, identifier)

## Translation -> migrations/0001_fixed_tables.sql
id                  INTEGER
name                TEXT                Original, Translated

## Language -> migrations/0001_fixed_tables.sql
id                  INTEGER
code                TEXT

## Lyrics Variant -> migrations/0003_lyrics_tables.sql
id                  INTEGER
translation_id      INTEGER
language_id         INTEGER
UNIQUE(translation_id, language_id)

## Lyrics Format -> migrations/0001_fixed_tables.sql
id                  INTEGER
format              TEXT                e.g. lrc, text or elrc

## Lyrics File -> migrations/0003_lyrics_tables.sql
uuid                UUID
recording_uuid      Index(UUID)
variant_id          INTEGER
format_id           INTEGER
file_path           TEXT
UNIQUE(recording_id, variant_id, format_id)

## Audio File -> migrations/0003_lyrics_tables.sql
uuid                UUID
recording_uuid      Index(UUID)
file_path           TEXT
