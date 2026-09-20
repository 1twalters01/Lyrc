from dataclasses import dataclass
from aligner.abstractions.options import AlignmentOptions
from aligner.models.language import Language

@dataclass
class WhisperXCueOptions(AlignmentOptions):
    language: Language
    model: str = "large-v3"

