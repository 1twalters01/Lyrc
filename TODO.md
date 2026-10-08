# V1
## Todo
* Have a proper config module
    - [ ] Read from toml/json file 

* Add download providers
    - [ ] Test Musixmatch

* Have defined requirements

* Add error types to python libraries instead of just raising an error

* Improve UI/UX/Design
    * Improve keyboard commands
    * Signify that you are editing a cue (maybe add a cursor?)

## Release Criteria
- [x] Reliably detects the current track
- [ ] Can download lyrics of current track
    - [x] Can choose the download provider
    - [x] From LrcLib
    - [ ] From Musixmatch
- [x] Can force-align lyrics
    - [x] Cue level to Word level alignment
    - [x] No alignment to Cue level alignment ---> This is currently scuffed
    - [x] Can choose the alignment to switch to whilst in the app
- [x] Can Translate lyrics 
    - [x] Can use Argos translator to translate lyrics
    - [x] Can switch the displayed lyrics language
    - [x] Can choose the language to switch to whilst in the app
- [x] Can automatically read the best aligned file type for a track (elrc, lrc, txt)
- [ ] Can save and reload lyrics correctly
    - [x] Can save subtitles
    - [x] Can reload subtitles
    - [x] Can choose where save subtitles
    - [x] Save where the file was saved in a db
    - [ ] Can retrieve a subtitle file from where it was saved
- [x] Can edit lyrics
    - [x] Can edit lyrics of untimed lines
    - [x] Can edit lyrics of line aligned cues
    - [x] Can edit lyrics of word aligned cues
    - [x] Adding a space creates a new word in word aligned cues
    - [x] Backspace on first letter of non-first word fuses the words together
    - [x] Delete on last letter of non-last word fuses the words together
    - [x] Insert / delete line / edit text / edit time stamp
    - [x] Undo / redo
    - [x] Preserve alignment when text changes
- [x] Mpris
    - [x] Play/Pause/Seek
    - [x] Can select and go to the time of a given lyric
    - [x] Can select and go to the time of a given word if word aligned
    - [x] Modal to select a player
- [x] Can handle empty lines (e.g. new lines)
- [ ] Handles errors gracefully
    - [ ] Handles errors from rust without crashing
    - [ ] Handles errors from python without crashing
- [ ] Test everything
- [ ] Write requirements and fully go through them
- [ ] Good UI/UX
    - [ ] Good Keyboard commands
    - [ ] Good Design


# V2
* GUI(s)
    * Linux - GTK4
    * MacOS?
    * Windows?
* CLI Commands?
* Improve cue level alignment
* Refactor
* Add Genius as a download option
* Translate lyrics
    [ ] Google
        [ ] Fill in options python variable in rust provider file
        [ ] Test
    [ ] DeepL
        [ ] Fill in options python variable in rust provider file
        [ ] Test
    [ ] Huggingface
        [ ] Fill in options python variable in rust provider file
        [ ] Test
    [ ] Ollama
        [ ] Fill in options python variable in rust provider file
        [ ] Test
* Handle event for change in playback speed
- [ ] Handle changing playback speed
- [ ] Split/merge lines


# V3
* App Daemon - Elixir/Gleam
* Android app
    * PC Daemon
    * Cross-platform vs native
* Add Kugou/NetEase as download options
