# Schema
Field               Type                Description

## Artist
id                  UUID
name                TEXT

## Song
id                  UUID
title               TEXT
artist_id           UUID

## Recording
id                  UUID
song_id             Index(UUID)
artist_id           Index(UUID)         Could be a cover so artist needed here as well
duration_ms         INTEGER

## Source -> migrations/0001_fixed_tables.sql
id                  UUID
source              TEXT                e.g. musicbrainz, or liblrc

## Recording Identifier
id                  UUID
recording_id        Index(UUID)
source_id           UUID                Source of the recording identifier
identifier          TEXT                Identifier from the source
UNIQUE(source_id, identifier)

## Translation -> migrations/0001_fixed_tables.sql
id                  INTEGER
name                TEXT                Original, Translated

## Language -> migrations/0001_fixed_tables.sql
id                  UUID
code                TEXT

## Lyrics Variant 
id                  UUID
translation_id      INTEGER
language_id         UUID
UNIQUE(translation_id, language_id)

## Lyrics Format -> migrations/0001_fixed_tables.sql
id                  UUID
format              TEXT                e.g. lrc, text or elrc

## Lyrics File
id                  UUID
recording_id        Index(UUID)
variant_id          UUID
format_id           UUID
path                TEXT
UNIQUE(recording_id, variant_id, format_id)
