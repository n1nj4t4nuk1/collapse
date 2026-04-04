"""
7z compression strategy using the LZMA2 algorithm.

Maps API compression levels (1–5) to py7zr presets and activates
PRESET_EXTREME at the maximum level for the greatest size reduction.
"""

import py7zr

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy

# Mapping: API level → py7zr preset (1 = minimum, 9 = maximum)
_SEVENZ_PRESETS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class SevenZipCompressionStrategy(CompressionStrategy):
    """Compresses files into .7z archives using the py7zr LZMA2 filter."""

    @property
    def algorithm(self) -> CompressionAlgorithm:
        return CompressionAlgorithm.SEVENZ

    def compress(self, source_path, archive_path, original_filename: str, level: int) -> None:
        """
        Create a .7z archive containing the source file.

        At level 5, the maximum preset is combined with ``PRESET_EXTREME``
        for the highest compression ratio at the cost of additional CPU time.
        """
        preset = _SEVENZ_PRESETS[level]
        if level == 5:
            preset |= py7zr.PRESET_EXTREME
        filters = [{"id": py7zr.FILTER_LZMA2, "preset": preset}]

        with py7zr.SevenZipFile(archive_path, mode="w", filters=filters) as archive:
            archive.write(source_path, arcname=original_filename)
