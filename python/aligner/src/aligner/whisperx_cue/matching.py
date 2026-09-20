from dataclasses import dataclass
from datetime import timedelta
from difflib import SequenceMatcher
import unicodedata

import numpy as np
import whisperx

from aligner.abstractions.providers import AlignmentProvider
from aligner.models.aligned_cue import AlignedCue, Word
from aligner.models.cue import Cue
from aligner.whisperx_cue.options import WhisperXCueOptions


@dataclass(frozen=True)
class TranscriptWord:
    text: str
    line_index: int


@dataclass(frozen=True)
class WordAlignment:
    transcript_index: int | None
    whisper_index: int | None


@dataclass(frozen=True)
class LineMatch:
    line_index: int
    start: timedelta
    end: timedelta
    matched_words: int
    total_words: int

    @property
    def coverage(self) -> float:
        if self.total_words == 0:
            return 0.0

        return self.matched_words / self.total_words


def normalize_word(word: str) -> str:
    word = unicodedata.normalize("NFD", word)

    return "".join(
        char.lower()
        for char in word
        if char.isalnum()
    )


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
        dp[i][0] = dp[i - 1][0] + DELETION_COST

    for j in range(1, whisper_count + 1):
        dp[0][j] = dp[0][j - 1] + INSERTION_COST

    for i in range(1, transcript_count + 1):
        for j in range(1, whisper_count + 1):
            substitution = (
                dp[i - 1][j - 1]
                + substitution_cost(
                    transcript[i - 1].text,
                    whisper[j - 1].text,
                )
            )

            deletion = (
                dp[i - 1][j]
                + DELETION_COST
            )

            insertion = (
                dp[i][j - 1]
                + INSERTION_COST
            )

            dp[i][j] = min(
                substitution,
                deletion,
                insertion,
            )

    alignment: list[WordAlignment] = []

    i = transcript_count
    j = whisper_count

    while i > 0 or j > 0:

        # Substitution / exact match
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

        # Deletion
        if (
            i > 0
            and dp[i][j]
            == dp[i - 1][j] + DELETION_COST
        ):
            alignment.append(
                WordAlignment(
                    transcript_index=i - 1,
                    whisper_index=None,
                )
            )

            i -= 1
            continue

        # Insertion
        if (
            j > 0
            and dp[i][j]
            == dp[i][j - 1] + INSERTION_COST
        ):
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

    words: list[TranscriptWord] = []

    for line_index, line in enumerate(lines):
        for word in line.split():
            words.append(
                TranscriptWord(
                    text=word,
                    line_index=line_index,
                )
            )

    return words


def build_line_matches(
    transcript: list[TranscriptWord],
    whisper: list[Word],
    alignment: list[WordAlignment],
) -> list[LineMatch]:
    line_words: dict[int, list[Word]] = {}
    line_total_words: dict[int, int] = {}

    for word in transcript:
        line_total_words[word.line_index] = (
            line_total_words.get(word.line_index, 0)
            + 1
        )

    for item in alignment:
        if item.transcript_index is None:
            continue

        if item.whisper_index is None:
            continue

        transcript_word = transcript[
            item.transcript_index
        ]

        whisper_word = whisper[
            item.whisper_index
        ]

        line_words.setdefault(
            transcript_word.line_index,
            [],
        ).append(whisper_word)

    matches: list[LineMatch] = []

    for line_index in sorted(line_words):

        words = line_words[line_index]

        if not words:
            continue

        total_words = line_total_words[line_index]

        matches.append(
            LineMatch(
                line_index=line_index,
                start=min(
                    word.start
                    for word in words
                ),
                end=max(
                    word.end
                    for word in words
                ),
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

        alignment = align_words(
            transcript,
            whisper_words,
        )

        return build_line_matches(
            transcript,
            whisper_words,
            alignment,
        )
