import os
import pytest

from .lrc_contents import LRC_CONTENTS
from aligner.models.language import Language
from aligner.service import AlignmentService
from aligner.whisperx_word.provider import WhisperXWordAligner
from aligner.whisperx_word.options import WhisperXWordOptions

def test_whisperx_word_aligner():
    audio_path = os.getenv("WHISPERX_TEST_AUDIO")
    if audio_path is None:
        pytest.skip("WHISPERX_TEST_AUDIO not set")

    lrc_contents = LRC_CONTENTS
    language = Language(
        name="Spanish",
        native_name="Español",
        code_2="es",
        code_3="spa",
        flores_200="spa_Latn",
    )
    device = "cuda"

    service = AlignmentService({
        "whisperx_word": WhisperXWordAligner(device=device),
    })
    options = WhisperXWordOptions(language=language)
    aligned_cues = service.align_cues(
        provider_name="whisperx_word",
        content=lrc_contents,
        audio_path=audio_path,
        options=options
    )

    # print(aligned_cues)
