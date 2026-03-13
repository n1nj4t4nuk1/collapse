import py7zr

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy

_SEVENZ_PRESETS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class SevenZipCompressionStrategy(CompressionStrategy):
    @property
    def algorithm(self) -> CompressionAlgorithm:
        return CompressionAlgorithm.SEVENZ

    def compress(self, source_path, archive_path, original_filename: str, level: int) -> None:
        preset = _SEVENZ_PRESETS[level]
        if level == 5:
            preset |= py7zr.PRESET_EXTREME
        filters = [{"id": py7zr.FILTER_LZMA2, "preset": preset}]

        with py7zr.SevenZipFile(archive_path, mode="w", filters=filters) as archive:
            archive.write(source_path, arcname=original_filename)
