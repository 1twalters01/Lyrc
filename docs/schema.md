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

## Source
id                  UUID
name                TEXT                e.g. musicbrainz, or liblrc

## Recording Identifier
id                  UUID
recording_id        Index(UUID)
source_id           UUID
identifier          TEXT                Identifier from the source
UNIQUE(source_id, identifier)

## Translation
id                  UUID
type                TEXT                Original, Translated

## Language
id                  UUID
language            TEXT

## Lyrics Variant
id                  UUID
translation_id      UUID
language_id         UUID
UNIQUE(translation_id, language_id)

## Lyrics Format
id                  UUID
format              TEXT                e.g. lrc, text or elrc

## Lyrics File
id                  UUID
recording_id        Index(UUID)
variant_id          UUID
format_id           UUID
path                TEXT
UNIQUE(recording_id, variant_id, format_id)
