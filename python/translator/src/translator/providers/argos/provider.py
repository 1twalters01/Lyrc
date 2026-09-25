from translator.models.cue import Cue
from translator.models.language import Language
from translator.abstractions.providers import TranslationProvider
from translator.providers.argos.options import ArgosOptions

import argostranslate.package
import argostranslate.translate


# Mute logs
import logging
import warnings
def configure_quiet_mode() -> None:
    logging.disable(logging.CRITICAL)
    warnings.filterwarnings("ignore")

configure_quiet_mode()


class ArgosTranslator(TranslationProvider[ArgosOptions]):
    def __init__(self):
        pass

    def translate(
            self,
            cues: list[Cue],
            to_language: Language,
            options: ArgosOptions,
    ) -> list[Cue]:
        from_code = options.original_language.code_2
        to_code = to_language.code_2

        installed_languages = argostranslate.translate.get_installed_languages()
        path = ArgosTranslator.find_installed_path(installed_languages, from_code, to_code)

        if path is None:
            raise ValueError(
                f"No installed Argos translation path found: "
                    f"{from_code} -> {to_code}"
            )

        translated_cues: list[Cue] = []
        for cue in cues:
            result = ArgosTranslator.translate_along_path(cue.content, path)
            translated_cues.append(
                Cue(
                    start=cue.start,
                    end=cue.end,
                    content=result,
                )
            )

        return translated_cues
         
    @staticmethod
    def find_installed_path(
        installed_languages,
        from_code: str,
        to_code: str,
    ) -> list[str] | None:
        graph: dict[str, list[str]] = {}

        for language in installed_languages:
            graph[language.code] = [
                translation.to_lang.code
                for translation in language.translations_from
            ]

        queue = [(from_code, [from_code])]
        visited = {from_code}

        while queue:
            current, path = queue.pop(0)

            if current == to_code:
                return path

            for next_code in graph.get(current, []):
                if next_code not in visited:
                    visited.add(next_code)
                    queue.append(
                        (next_code, path + [next_code])
                    )

        return None
 
    @staticmethod
    def translate_along_path(text: str, path: list[str]) -> str:
        for from_code, to_code in zip(path, path[1:]):
            text = argostranslate.translate.translate(
                text,
                from_code,
                to_code,
            )

        return text
 
    @staticmethod
    def install_packages(to_language: Language, from_language: Language):
        to_code = to_language.code_2
        from_code = from_language.code_2

        installed_langauges = argostranslate.translate.get_installed_languages()

        installed_path = ArgosTranslator.find_installed_path(
            installed_langauges,
            from_code,
            to_code,
        )

        if installed_path is not None:
            return

        argostranslate.package.update_package_index()
        available_packages = argostranslate.package.get_available_packages()

        path = ArgosTranslator.find_path(available_packages, from_code, to_code)
        if path is None:
            raise ValueError(
                f"No Argos translation path found: "
                f"{from_code} -> {to_code}"
            )

        ArgosTranslator.install_path(path)

    @staticmethod
    def find_path(
        available_packages,
        from_code: str,
        to_code: str,
    ) -> list[str] | None:
        graph: dict[str, list[str]] = {}

        for package in available_packages:
            graph.setdefault(package.from_code, []).append(package.to_code)

        queue = [(from_code, [from_code])]
        visited = {from_code}

        while queue:
            current, path = queue.pop(0)

            if current == to_code:
                return path

            for next_code in graph.get(current, []):
                if next_code not in visited:
                    visited.add(next_code)
                    queue.append((next_code, path + [next_code]))

        return None

    @staticmethod
    def install_path(path: list[str]):
        argostranslate.package.update_package_index()
        available_packages = argostranslate.package.get_available_packages()

        for from_code, to_code in zip(path, path[1:]):
            package = next(
                (
                    package
                    for package in available_packages
                    if package.from_code == from_code
                    and package.to_code == to_code
                ),
                None,
            )

            if package is None:
                raise ValueError(
                    f"No package found for {from_code} -> {to_code}"
                )

            argostranslate.package.install_from_path(package.download())
