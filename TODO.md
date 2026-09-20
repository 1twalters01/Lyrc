# V1
## Todo
* Have a proper config module
    [ ] Read from toml/json file 

* Add download providers
    [ ] Test Musixmatch

* Have defined requirements

* Allow language switching on the fly

* Add error types to python libraries instead of just raising an error

* Improve UI/UX/Design
    * Improve keyboard commands
    * Signify that you are editing a cue (maybe add a cursor?)

## Release Criteria
- [x] Reliably detects the current track
- [ ] Can download lyrics of current track
    - [x] From LrcLib
    - [ ] From Musixmatch
- [x] Can force-align lyrics
    - [x] Cue level to Word level alignment
    - [x] No alignment to Cue level alignment ---> This is currently scuffed
- [ ] Can Translate lyrics 
    - [x] Can use Argos translator to translate lyrics
    - [ ] Can switch the displayed lyrics language
- [x] Can automatically read the best aligned file type for a track (elrc, lrc, txt)
- [ ] Can save and reload lyrics correctly
    - [x] Can save subtitles
    - [x] Can reload subtitles
    - [ ] Can choose where and how to save subtitles
    - [ ] Save where the file was saved in a db and retrieve based on the track
- [ ] Handle changing playback speed
- [ ] Can edit lyrics
    - [x] Can edit lyrics of untimed lines
    - [x] Can edit lyrics of line aligned cues
    - [ ] Can edit lyrics of word aligned cues
    - [x] Insert / delete line / edit text / edit time stamp
    - [x] Undo / redo
    - [x] Preserve alignment when text changes
    - [ ] Split/merge lines
- [ ] Mpris
    - [x] Play/Pause/Seek
    - [x] Can select and go to the time of a given lyric
    - [ ] Can select and go to the time of a given word if word aligned
- [ ] Can handle empty lines (e.g. new lines)
- [ ] Handles errors gracefully
    - [x] Handles errors from rust without crashing
    - [ ] Handles errors from python without crashing
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


# V3
* App Daemon - Elixir/Gleam
* Android app
    * PC Daemon
    * Cross-platform vs native
* Add Kugou/NetEase as download options
