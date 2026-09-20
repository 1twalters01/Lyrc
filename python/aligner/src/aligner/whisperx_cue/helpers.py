from datetime import timedelta

from aligner.models.aligned_cue import Word
from aligner.whisperx_cue.localisation import LineLocalization, LocalizationRegion
from aligner.whisperx_cue.matching import LineMatch, TranscriptMatcher


MIN_ANCHOR_WORDS = 2
MIN_ANCHOR_COVERAGE = 0.5

def is_strong_match(match: LineMatch) -> bool:
    return (
        match.matched_words >= MIN_ANCHOR_WORDS
        and match.coverage >= MIN_ANCHOR_COVERAGE
        and match.end > match.start
    )

def match_to_localization(
    match: LineMatch,
) -> LineLocalization:
    return LineLocalization(
        line_index=match.line_index,
        start=match.start,
        end=match.end,
        coverage=match.coverage,
    )

def words_in_region(
    words: list[Word],
    start: timedelta,
    end: timedelta,
) -> list[Word]:
    return [
        word
        for word in words
        if word.end >= start
        and word.start <= end
    ]

def select_anchors(
    matches: list[LineMatch],
) -> list[LineMatch]:
    anchors = [
        match
        for match in matches
        if is_strong_match(match)
    ]

    anchors.sort(
        key=lambda match: match.line_index
    )

    return anchors
