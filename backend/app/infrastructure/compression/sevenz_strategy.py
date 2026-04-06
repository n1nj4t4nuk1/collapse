"""7z compression adapter using the LZMA2 algorithm.

Maps API compression levels (1-5) to py7zr presets and activates
PRESET_EXTREME at the maximum level for the greatest size reduction.
"""

from pathlib import Path

import py7zr

from app.domain.models.job import CompressionAlgorithm
from app.domain.ports.compression_strategy import CompressionStrategyPort

# Mapping: API level -> py7zr preset (1 = minimum, 9 = maximum)
_SEVENZ_PRESETS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class SevenZipCompression(CompressionStrategyPort):
    """Compresses files into .7z archives using the py7zr LZMA2 filter."""

    @property
    def algorithm(self) -> CompressionAlgorithm:
        return CompressionAlgorithm.SEVENZ

    def compress(
        self,
        source_path: Path,
        archive_path: Path,
        original_filename: str,
        level: int,
    ) -> None:
        preset = _SEVENZ_PRESETS[level]
        if level == 5:
            preset |= py7zr.PRESET_EXTREME
        filters = [{"id": py7zr.FILTER_LZMA2, "preset": preset}]

        with py7zr.SevenZipFile(archive_path, mode="w", filters=filters) as archive:
            archive.write(source_path, arcname=original_filename)
