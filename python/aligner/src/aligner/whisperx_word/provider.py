from datetime import timedelta
import whisperx
from aligner.models.cue import Cue
from aligner.models.aligned_cue import AlignedCue, Word
from aligner.abstractions.providers import AlignmentProvider
from aligner.whisperx_word.options import WhisperXWordOptions

class WhisperXWordAligner(AlignmentProvider[WhisperXWordOptions]):
    def __init__(self, device: str):
        self.device = device

    def align_cues(
        self,
        content: list[Cue | str],
        audio_path: str,
        options: WhisperXWordOptions
    ) -> list[AlignedCue | Cue]:
        language_code = options.language.code_2
        device = self.device

        segments = [
            {
                "start": cue.start.total_seconds(),
                "end": cue.end.total_seconds(),
                "text": cue.content.strip()
            }
            for cue in content
            if cue.content.strip()
        ]
        if not segments:
            raise ValueError("No content found")

        audio = whisperx.load_audio(audio_path)
        model_a, metadata = whisperx.load_align_model(language_code=language_code, device=device)
        result = whisperx.align(segments, model_a, metadata, audio, device, return_char_alignments=False)

        aligned_cues: List[AlignedCue] = []
        for index, segment in enumerate(result["segments"]):
            segment_start = timedelta(
                seconds=min(segments[index].get("start"), segment["start"])
            )
            segment_end = timedelta(
                seconds=max(segments[index].get("end"), segment["end"])
            )
            segment_words = segment.get("words", [])
            # print(f'\n{segment["start"]} -> {segment["end"]}')
            # print(f"\n{segment_start} -> {segment_end}")

            words: List[Word] = []
            for segment_word in segment_words:
                word_start = timedelta(seconds=segment_word.get("start", segment_start))
                word_end = timedelta(seconds=segment_word.get("end", segment_end))
                word_text = segment_word.get("word", "").strip()
                # print(f"    {word_start} -> {word_end}: {word_text}")
                
                words.append(
                    Word(
                        start=word_start,
                        end=word_end,
                        text=word_text,
                    )
                )

            aligned_cues.append(
                AlignedCue(
                    start=segment_start,
                    end=segment_end,
                    words=words
                )
            )

        return aligned_cues
