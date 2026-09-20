from dataclasses import dataclass
from datetime import timedelta
# from aligner.whisperx_cue.helpers import is_strong_match, match_to_localization, words_in_region

@dataclass(frozen=True)
class LineLocalization:
    line_index: int
    start: timedelta
    end: timedelta
    coverage: float

    @property
    def duration(self) -> timedelta:
        return self.end - self.start

    @property
    def is_anchor(self) -> bool:
        """
        Whether this localization is trustworthy enough to constrain
        neighbouring transcript lines.
        """

        return (
            self.coverage >= 0.5
            and self.end > self.start
        )


@dataclass(frozen=True)
class LocalizationRegion:
    """
    Transcript lines [start_line, end_line) occupy the audio region
    [start, end].
    """

    start_line: int
    end_line: int
    start: timedelta
    end: timedelta

    @property
    def line_count(self) -> int:
        return self.end_line - self.start_line

