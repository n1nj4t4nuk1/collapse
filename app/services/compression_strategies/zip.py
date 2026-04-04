"""
ZIP compression strategy using the Deflate algorithm.

Maps API compression levels (1–5) to ``zipfile.ZipFile`` compression
levels (1–9).
"""

import zipfile

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy

# Mapping: API level → Deflate compression level (1 = minimum, 9 = maximum)
_ZIP_LEVELS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class ZipCompressionStrategy(CompressionStrategy):
    """Compresses files into .zip archives using the standard-library Deflate algorithm."""

    @property
    def algorithm(self) -> CompressionAlgorithm:
        return CompressionAlgorithm.ZIP

    def compress(self, source_path, archive_path, original_filename: str, level: int) -> None:
        """Create a .zip archive containing the source file using Deflate compression."""
        compresslevel = _ZIP_LEVELS[level]

        with zipfile.ZipFile(
            archive_path,
            mode="w",
            compression=zipfile.ZIP_DEFLATED,
            compresslevel=compresslevel,
        ) as archive:
            archive.write(source_path, arcname=original_filename)
