# Schema
| Table                    | Field                                           | Type        | Description                                        | Migration                |
| ------------------------ | ----------------------------------------------- | ----------- | -------------------------------------------------- | ------------------------ |
| **Artist**               | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `name`                                          | TEXT        |                                                    |                          |
| **Song**                 | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `title`                                         | TEXT        |                                                    |                          |
| **Song Artist**          | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `song_uuid`                                     | Index(UUID) |                                                    |                          |
|                          | `artist_uuid`                                   | UUID        |                                                    |                          |
| **Recording**            | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `song_uuid`                                     | Index(UUID) |                                                    |                          |
|                          | `duration_ms`                                   | INTEGER     |                                                    |                          |
| **Recording Artist**     | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `recording_uuid`                                | Index(UUID) |                                                    |                          |
|                          | `artist_uuid`                                   | Index(UUID) | Could be a cover, so artist is needed here as well |                          |
| **Source**               | `id`                                            | INTEGER     |                                                    | `0001_fixed_tables.sql`  |
|                          | `source`                                        | TEXT        | e.g. MusicBrainz or liblrc                         |                          |
| **Recording Identifier** | `uuid`                                          | UUID        |                                                    | `0002_core_tables.sql`   |
|                          | `recording_uuid`                                | Index(UUID) |                                                    |                          |
|                          | `source_id`                                     | INTEGER     | Source of the recording identifier                 |                          |
|                          | `identifier`                                    | TEXT        | Identifier from the source                         |                          |
|                          | `UNIQUE(source_id, identifier)`                 |             |                                                    |                          |
| **Translation**          | `id`                                            | INTEGER     |                                                    | `0001_fixed_tables.sql`  |
|                          | `name`                                          | TEXT        | Original, Translated                               |                          |
| **Language**             | `id`                                            | INTEGER     |                                                    | `0001_fixed_tables.sql`  |
|                          | `code_3`                                        | TEXT        | ISO 639-3 code                                     |                          |
| **Lyrics Variant**       | `id`                                            | INTEGER     |                                                    | `0003_file_tables.sql`    |
|                          | `translation_id`                                | INTEGER     |                                                    |                          |
|                          | `language_id`                                   | INTEGER     |                                                    |                          |
|                          | `UNIQUE(translation_id, language_id)`           |             |                                                    |                          |
| **Lyrics Format**        | `id`                                            | INTEGER     |                                                    | `0001_fixed_tables.sql`  |
|                          | `format`                                        | TEXT        | e.g. LRC, text or ELRC                             |                          |
| **Lyrics File**          | `uuid`                                          | UUID        |                                                    | `0003_file_tables.sql` |
|                          | `recording_uuid`                                | Index(UUID) |                                                    |                          |
|                          | `variant_id`                                    | INTEGER     |                                                    |                          |
|                          | `format_id`                                     | INTEGER     |                                                    |                          |
|                          | `file_path`                                     | TEXT        |                                                    |                          |
|                          | `UNIQUE(recording_uuid, variant_id, format_id)` |             |                                                    |                          |
| **Audio File**           | `uuid`                                          | UUID        |                                                    | `0003_file_tables.sql` |
|                          | `recording_uuid`                                | Index(UUID) |                                                    |                          |
|                          | `file_path`                                     | TEXT        |                                                    |                          |
