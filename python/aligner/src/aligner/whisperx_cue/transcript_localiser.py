from aligner.whisperx_cue.helpers import is_strong_match, match_to_localization, words_in_region, select_anchors
from aligner.whisperx_cue.localisation import LineLocalization, LocalizationRegion
from aligner.whisperx_cue.matching import LineMatch, TranscriptMatcher
from aligner.models.aligned_cue import Word

def find_regions(
    anchors: list[LineMatch],
) -> list[LocalizationRegion]:
    regions: list[LocalizationRegion] = []

    for previous, next_ in zip(
        anchors,
        anchors[1:],
    ):
        # Adjacent transcript lines don't leave a gap.
        if next_.line_index <= previous.line_index + 1:
            continue

        regions.append(
            LocalizationRegion(
                start_line=previous.line_index + 1,
                end_line=next_.line_index,
                start=previous.end,
                end=next_.start,
            )
        )

    return regions


def match_region(
    region: LocalizationRegion,
    lines: list[str],
    whisper_words: list[Word],
    matcher: TranscriptMatcher,
) -> list[LineMatch]:
    region_lines = lines[
        region.start_line:
        region.end_line
    ]

    region_words = words_in_region(
        whisper_words,
        region.start,
        region.end,
    )

    if not region_words:
        return []

    matches = matcher.match(
        region_lines,
        region_words,
    )

    # Convert relative line indices back to absolute transcript indices.
    return [
        LineMatch(
            line_index=(
                match.line_index
                + region.start_line
            ),
            start=match.start,
            end=match.end,
            matched_words=match.matched_words,
            total_words=match.total_words,
        )
        for match in matches
    ]


def provisional_localizations(
    region: LocalizationRegion,
) -> list[LineLocalization]:
    if region.line_count <= 0:
        return []

    duration = region.end - region.start
    step = duration / region.line_count

    result: list[LineLocalization] = []

    for offset in range(region.line_count):
        start = region.start + step * offset
        end = region.start + step * (offset + 1)

        result.append(
            LineLocalization(
                line_index=region.start_line + offset,
                start=start,
                end=end,
                coverage=0.0,
            )
        )

    return result


class TranscriptLocalizer:
    def __init__(
        self,
        matcher: TranscriptMatcher,
    ):
        self.matcher = matcher

    def localize(
        self,
        lines: list[str],
        whisper_words: list[Word],
    ) -> list[LineLocalization]:
        global_matches = self.matcher.match(
            lines,
            whisper_words,
        )

        anchors = select_anchors(
            global_matches
        )

        localizations = {
            anchor.line_index:
                match_to_localization(anchor)
            for anchor in anchors
        }

        regions = self._build_initial_regions(
            lines,
            anchors,
            whisper_words,
        )

        for region in regions:
            self._resolve_region(
                region=region,
                lines=lines,
                whisper_words=whisper_words,
                localizations=localizations,
            )

        return sorted(
            localizations.values(),
            key=lambda localization:
                localization.line_index,
        )

    def _build_initial_regions(
        self,
        lines: list[str],
        anchors: list[LineMatch],
        whisper_words: list[Word],
    ) -> list[LocalizationRegion]:
        regions: list[LocalizationRegion] = []

        # Before first anchor
        if anchors:
            first = anchors[0]

            if first.line_index > 0:

                regions.append(
                    LocalizationRegion(
                        start_line=0,
                        end_line=first.line_index,
                        start=timedelta(0),
                        end=first.start,
                    )
                )

        # Between anchors
        regions.extend(
            find_regions(anchors)
        )

        # After last anchor
        if anchors:
            last = anchors[-1]

            if last.line_index < len(lines) - 1:
                if whisper_words:
                    audio_end = max(
                        word.end
                        for word in whisper_words
                    )

                    regions.append(
                        LocalizationRegion(
                            start_line=(
                                last.line_index + 1
                            ),
                            end_line=len(lines),
                            start=last.end,
                            end=audio_end,
                        )
                    )

        # No anchors
        elif whisper_words:

            audio_end = max(
                word.end
                for word in whisper_words
            )

            regions.append(
                LocalizationRegion(
                    start_line=0,
                    end_line=len(lines),
                    start=timedelta(0),
                    end=audio_end,
                )
            )

        return regions

    # Recursive region resolution
    def _resolve_region(
        self,
        region: LocalizationRegion,
        lines: list[str],
        whisper_words: list[Word],
        localizations: dict[int, LineLocalization],
    ) -> None:

        if region.line_count <= 0:
            return

        matches = match_region(
            region=region,
            lines=lines,
            whisper_words=whisper_words,
            matcher=self.matcher,
        )

        # --------------------------------------------------------------
        # IMPORTANT:
        #
        # Only strong matches are allowed to become anchors.
        #
        # This prevents things like:
        #
        #   20% 31.117 → 31.117
        #
        # from becoming a localization.
        # --------------------------------------------------------------

        anchors = [
            match
            for match in matches
            if is_strong_match(match)
        ]

        anchors.sort(
            key=lambda match: match.line_index
        )

        # Remove anchors that don't actually belong inside this region.
        anchors = [
            anchor
            for anchor in anchors
            if (
                region.start_line
                <= anchor.line_index
                < region.end_line
            )
        ]

        # --------------------------------------------------------------
        # No useful local anchors.
        #
        # We cannot determine individual positions from Whisper text,
        # so use broad provisional windows.
        # --------------------------------------------------------------

        if not anchors:
            for localization in (
                provisional_localizations(region)
            ):
                localizations[
                    localization.line_index
                ] = localization

            return

        # --------------------------------------------------------------
        # Add local anchors.
        # --------------------------------------------------------------

        for anchor in anchors:
            localization = (
                match_to_localization(anchor)
            )

            localizations[
                localization.line_index
            ] = localization

        # --------------------------------------------------------------
        # Recursively resolve the regions around the new anchors.
        # --------------------------------------------------------------

        boundaries: list[
            tuple[int, timedelta, timedelta]
        ] = []

        # Left side of region.
        first = anchors[0]
        if first.line_index > region.start_line:
            boundaries.append(
                (
                    region.start_line,
                    region.start,
                    first.start,
                )
            )

        # Between local anchors.
        for previous, next_ in zip(
            anchors,
            anchors[1:],
        ):
            if next_.line_index > previous.line_index + 1:
                boundaries.append(
                    (
                        previous.line_index + 1,
                        previous.end,
                        next_.start,
                    )
                )

        # Right side of region.
        last = anchors[-1]
        if last.line_index < region.end_line - 1:

            boundaries.append(
                (
                    last.line_index + 1,
                    last.end,
                    region.end,
                )
            )

        # Recurse.
        for start_line, start, end in boundaries:

            if start_line >= region.end_line:
                continue

            # Determine the first transcript line after start_line
            # that is still before the next anchor.
            next_anchor = next(
                (
                    anchor
                    for anchor in anchors
                    if anchor.line_index >= start_line
                ),
                None,
            )

            if next_anchor is not None:
                end_line = next_anchor.line_index
            else:
                end_line = region.end_line

            if end_line <= start_line:
                continue

            self._resolve_region(
                region=LocalizationRegion(
                    start_line=start_line,
                    end_line=end_line,
                    start=start,
                    end=end,
                ),
                lines=lines,
                whisper_words=whisper_words,
                localizations=localizations,
            )

