sequenceDiagram
    participant ComputerA
    participant ComputerB

ComputerA
song changes
get song file path
look up song file path in audio_file table
Get recording_uuid
Get all recording_identifiers with recording_uuid
Get the identifiers and source_ids for said recording_identifiers
Send (source_id, identifier) array to ComputerB

ComputerB
Receive (source_id, identifier) array from ComputerA
Look up in recording_identifier table
Get associated recording_uuid
Get all? associated lyrics files in the lyrics_file table
Get all? associated audio files in the audio_file table
Select which ones to pick
