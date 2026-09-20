from aligner.whisperx_word.provider import WhisperXWordAligner
from aligner.whisperx_cue.provider import WhisperXCueAligner
from aligner.aeneas.provider import AeneasAligner

PROVIDERS = [
    WhisperXWordAligner,
    WhisperXCueAligner,
    AeneasAligner,
]

