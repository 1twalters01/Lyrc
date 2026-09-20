from datetime import timedelta
import whisperx
from aligner.models.cue import Cue
from aligner.models.aligned_cue import AlignedCue, Word
from aligner.abstractions.providers import AlignmentProvider
from aligner.whisperx_cue.options import WhisperXCueOptions

from dataclasses import dataclass
@dataclass(frozen=True)
class TranscriptWord:
    text: str
    line_index: int

import unicodedata
def normalize_word(word: str) -> str:
    word = unicodedata.normalize("NFD", word)

    return "".join(
        char.lower()
        for char in word
        if char.isalnum()
    )

from difflib import SequenceMatcher
def substitution_cost(
    transcript_word: str,
    whisper_word: str,
) -> float:
    SIMILARITY_THRESHOLD = 0.6

    a = normalize_word(transcript_word)
    b = normalize_word(whisper_word)

    if a == b:
        return 0.0

    similarity = SequenceMatcher(None, a, b).ratio()

    if similarity < SIMILARITY_THRESHOLD:
        return 2.0

    return 1.0 - similarity

@dataclass(frozen=True)
class WordAlignment:
    transcript_index: int | None
    whisper_index: int | None

def align_words(
    transcript: list[TranscriptWord],
    whisper: list[Word],
) -> list[WordAlignment]:
    DELETION_COST = 1.0
    INSERTION_COST = 1.0

    transcript_count = len(transcript)
    whisper_count = len(whisper)

    dp = [
        [0.0] * (whisper_count + 1)
        for _ in range(transcript_count + 1)
    ]

    for i in range(1, transcript_count + 1):
        dp[i][0] = (
            dp[i - 1][0] + DELETION_COST
        )

    for j in range(1, whisper_count + 1):
        dp[0][j] = (
            dp[0][j - 1] + INSERTION_COST
        )

    for i in range(1, transcript_count + 1):
        for j in range(1, whisper_count + 1):
            substitution = (
                dp[i - 1][j - 1]
                + substitution_cost(
                    transcript[i-1].text,
                    whisper[j - 1].text,
                )
            )

            deletion = (dp[i - 1][j] + DELETION_COST)
            insertion = (dp[i][j - 1] + INSERTION_COST)
            dp[i][j] = min(substitution, deletion, insertion)

    alignment: list[WordAlignment] = []

    i = transcript_count
    j = whisper_count

    while i > 0 or j > 0:
        if i > 0 and j > 0:
            cost = substitution_cost(
                transcript[i - 1].text,
                whisper[j - 1].text,
            )

            if dp[i][j] == dp[i - 1][j - 1] + cost:
                alignment.append(
                    WordAlignment(
                        transcript_index=i - 1,
                        whisper_index=j - 1,
                    )
                )
                i -= 1
                j -= 1
                continue

        if i > 0 and dp[i][j] == dp[i - 1][j] + DELETION_COST:
            alignment.append(
                WordAlignment(
                    transcript_index=i - 1,
                    whisper_index=None,
                )
            )
            i -= 1
            continue

        if j > 0 and dp[i][j] == dp[i][j - 1] + INSERTION_COST:
            alignment.append(
                WordAlignment(
                    transcript_index=None,
                    whisper_index=j - 1,
                )
            )
            j -= 1
            continue

        raise RuntimeError("Invalid alignment state")

    alignment.reverse()
    return alignment

def split_transcript_words(
    lines: list[str],
) -> list[TranscriptWord]:
    tokens: list[TranscriptWord] = []

    for line_index, line in enumerate(lines):
        for word in line.split():
            tokens.append(
                TranscriptWord(
                    text=word,
                    line_index=line_index,
                )
            )

    return tokens

@dataclass(frozen=True)
class LineMatch:
    line_index: int
    start: timedelta
    end: timedelta
    matched_words: int
    total_words: int

    @property
    def coverage(self) -> float:
        return self.matched_words / self.total_words

def build_line_matches(
    transcript: list[TranscriptWord],
    whisper: list[Word],
    alignment: list[WordAlignment],
) -> list[LineMatch]:
    line_words: dict[int, list[Word]] = {}
    line_total_words: dict[int, int] = {}

    for word in transcript:
        line_total_words[word.line_index] = (
            line_total_words.get(word.line_index, 0) + 1
        )

    for item in alignment:
        if item.transcript_index is None:
            continue

        if item.whisper_index is None:
            continue

        transcript_word = transcript[item.transcript_index]
        whisper_word = whisper[item.whisper_index]

        line_words.setdefault(
            transcript_word.line_index,
            [],
        ).append(whisper_word)

    matches: list[LineMatch] = []

    for line_index in sorted(line_words):
        words = line_words[line_index]
        total_words = line_total_words[line_index]

        if not words:
            continue

        matches.append(
            LineMatch(
                line_index=line_index,
                start=words[0].start,
                end=words[-1].end,
                matched_words=len(words),
                total_words=total_words,
            )
        )

    return matches


class TranscriptMatcher:
    def match(
        self,
        lines: list[str],
        whisper_words: list[Word],
    ) -> list[LineMatch]:
        transcript = split_transcript_words(lines)
        alignment = align_words(transcript, whisper_words)
        
        return build_line_matches(
            transcript,
            whisper_words,
            alignment,
        )

import numpy as np
class WhisperXCueAligner(AlignmentProvider[WhisperXCueOptions]):
    def __init__(self, device: str):
        self.device = device
        self.matcher = TranscriptMatcher()

    def align_cues(
        self,
        content: list[Cue | str],
        audio_path: str,
        options: WhisperXCueOptions,
    ) -> list[AlignedCue | Cue]:
        PADDING = timedelta(seconds=0.5)
        lines = [
            item
            if isinstance(item, str)
            else item.content
            for item in content
        ]

        audio = whisperx.load_audio(audio_path)

        whisper_words = self._transcribe_words(audio, options)
        print("transcript lines:", len(lines))
        print("whisper words:", len(whisper_words))
        print("first transcript lines:", lines[:5])
        print("first whisper words:", whisper_words[:20])
        for index, word in enumerate(whisper_words):
            print(
                index,
                f"{word.start.total_seconds():.2f}",
                f"{word.end.total_seconds():.2f}",
                repr(word.text),
            )
        matches = self.matcher.match(lines, whisper_words)
        print("matches:", matches)
        model, metadata = whisperx.load_align_model(
            language_code=options.language.code_2,
            device=self.device,
        )

        cues: list[Cue] = []

        for match in matches:
            print(
                    f"line {match.line_index}: "
                    f"{match.matched_words}/{match.total_words} "
                    f"({match.coverage:.0%}) "
                    f"{match.start.total_seconds():.2f} → "
                    f"{match.end.total_seconds():.2f}"
                )
            start = max(timedelta(0), match.start - PADDING)
            end = match.end + PADDING
            line = lines[match.line_index]
            segment = {
                "start": start.total_seconds(),
                "end": end.total_seconds(),
                "text": line,
            }

            result = whisperx.align(
                [segment],
                model,
                metadata,
                audio,
                self.device,
                return_char_alignments=False,
            )

            aligned_segment = result["segments"][0]

            cues.append(
                Cue(
                    start=timedelta(seconds=aligned_segment["start"]),
                    end=timedelta(seconds=aligned_segment["end"]),
                    content=line,
                )
            )

        print("\n\nresults: ", cues)
        return cues

    # def _transcribe_words(
    #     self,
    #     audio: np.ndarray,
    #     options: WhisperXCueOptions
    # ) -> list[Word]:
    #     model = whisperx.load_model(
    #         options.model,
    #         self.device,
    #         language=options.language.code_2,
    #     )
    #
    #     result = model.transcribe(
    #         audio,
    #         language=options.language.code_2,
    #     )
    #     print("segments:", len(result["segments"]))
    #     print("result:", result)
    #     words: list[Word] = []
    #
    #     for segment in result["segments"]:
    #         for word in segment.get("words", []):
    #             if "start" not in word or "end" not in word:
    #                 continue
    #
    #             text = word["word"].strip()
    #             if not text:
    #                 continue
    #
    #             words.append(
    #                 Word(
    #                     text=text,
    #                     start=word["start"],
    #                     end=word["end"],
    #                 )
    #             )
    #
    #     return words
    def _transcribe_words(
        self,
        audio: np.ndarray,
        options: WhisperXCueOptions,
    ) -> list[Word]:
        model = whisperx.load_model(
            options.model,
            self.device,
            language=options.language.code_2,
            vad_options={
                "vad_onset": 0.3,
                "vad_offset": 0.2,
            },
        )

        result = model.transcribe(
            audio,
            language=options.language.code_2,
        )

        align_model, metadata = whisperx.load_align_model(
            language_code=options.language.code_2,
            device=self.device,
        )

        result = whisperx.align(
            result["segments"],
            align_model,
            metadata,
            audio,
            self.device,
            return_char_alignments=False,
        )

        words = []

        for segment in result["segments"]:
            print(
                    f"{segment['start']:.2f} → {segment['end']:.2f}",
                    repr(segment["text"]),
                )
            for segment_word in segment.get("words", []):
                words.append(
                    Word(
                        start=timedelta(seconds=segment_word["start"]),
                        end=timedelta(seconds=segment_word["end"]),
                        text=segment_word["word"].strip(),
                    )
                )

        return words
