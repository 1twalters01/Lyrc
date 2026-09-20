import os
import pytest

from .txt_contents import TXT_CONTENTS
from aligner.models.language import Language
from aligner.service import AlignmentService
from aligner.whisperx_cue.provider import WhisperXCueAligner
from aligner.whisperx_cue.options import WhisperXCueOptions

def test_whisperx_cue_aligner():
    audio_path = os.getenv("WHISPERX_TEST_AUDIO")
    if audio_path is None:
        pytest.skip("WHISPERX_TEST_AUDIO not set")

    txt_contents = TXT_CONTENTS
    language = Language(
        name="Spanish",
        native_name="Español",
        code_2="es",
        code_3="spa",
        flores_200="spa_Latn",
    )
    device = "cuda"

    service = AlignmentService({
        "whisperx_cue": WhisperXCueAligner(device=device),
    })
    options = WhisperXCueOptions(language=language)
    aligned_cues = service.align_cues(
        provider_name="whisperx_cue",
        content=txt_contents,
        audio_path=audio_path,
        options=options
    )

    print("\n\nresults: ", aligned_cues)

