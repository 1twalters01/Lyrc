from dataclasses import dataclass
from datetime import timedelta
from difflib import SequenceMatcher
import unicodedata

import numpy as np
import whisperx

from aligner.abstractions.providers import AlignmentProvider
from aligner.models.aligned_cue import Word
from aligner.models.cue import Cue
from aligner.whisperx_cue.options import WhisperXCueOptions

from aligner.whisperx_cue.helpers import is_strong_match, match_to_localization, words_in_region, select_anchors
from aligner.whisperx_cue.matching import LineMatch, TranscriptMatcher
from aligner.whisperx_cue.localisation import LineLocalization, LocalizationRegion

from aligner.whisperx_cue.transcript_localiser import TranscriptLocalizer



# Mute logs
import logging
import warnings
def configure_quiet_mode() -> None:
    logging.disable(logging.CRITICAL)
    warnings.filterwarnings("ignore")

configure_quiet_mode()


class WhisperXCueAligner(
    AlignmentProvider[WhisperXCueOptions]
):
    def __init__(self, device: str):
        self.device = device
        self.matcher = TranscriptMatcher()
        self.localizer = TranscriptLocalizer(self.matcher)

    def align_cues(
        self,
        content: list[Cue | str],
        audio_path: str,
        options: WhisperXCueOptions,
    ) -> list[Cue]:
        PADDING = timedelta(seconds=0.5)

        lines = [
            item
            if isinstance(item, str)
            else item.content
            for item in content
        ]

        audio = whisperx.load_audio(audio_path)

        whisper_words = self._transcribe_words(audio, options)

        localizations = self.localizer.localize(
            lines=lines,
            whisper_words=whisper_words,
        )

        model, metadata = (
            whisperx.load_align_model(
                language_code=options.language.code_2,
                device=self.device,
            )
        )

        segments = []
        for localization in localizations:
            start = max(timedelta(0), localization.start - PADDING)
            end = localization.end + PADDING

            segments.append(
                {
                    "start": start.total_seconds(),
                    "end": end.total_seconds(),
                    "text": lines[localization.line_index],
                }
            )

        result = whisperx.align(
            segments,
            model,
            metadata,
            audio,
            self.device,
            return_char_alignments=False,
        )

        cues: list[Cue] = []
        for localization, segment in zip(
            localizations,
            result["segments"],
        ):
            start = segment.get(
                "start",
                localization.start.total_seconds(),
            )

            end = segment.get(
                "end",
                localization.end.total_seconds(),
            )

            cues.append(
                Cue(
                    start=timedelta(
                        seconds=start
                    ),
                    end=timedelta(
                        seconds=end
                    ),
                    content=lines[
                        localization.line_index
                    ],
                )
            )

        return cues

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

        align_model, metadata = (
            whisperx.load_align_model(
                language_code=options.language.code_2,
                device=self.device,
            )
        )

        result = whisperx.align(
            result["segments"],
            align_model,
            metadata,
            audio,
            self.device,
            return_char_alignments=False,
        )

        words: list[Word] = []
        for segment in result["segments"]:
            for segment_word in segment.get(
                "words",
                [],
            ):
                if (
                    "start" not in segment_word
                    or "end" not in segment_word
                ):
                    continue

                words.append(
                    Word(
                        start=timedelta(seconds=segment_word["start"]),
                        end=timedelta(seconds=segment_word["end"]),
                        text=segment_word["word"].strip(),
                    )
                )

        return words
