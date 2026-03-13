import zipfile

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.base import CompressionStrategy

_ZIP_LEVELS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class ZipCompressionStrategy(CompressionStrategy):
    @property
    def algorithm(self) -> CompressionAlgorithm:
        return CompressionAlgorithm.ZIP

    def compress(self, source_path, archive_path, original_filename: str, level: int) -> None:
        compresslevel = _ZIP_LEVELS[level]

        with zipfile.ZipFile(
            archive_path,
            mode="w",
            compression=zipfile.ZIP_DEFLATED,
            compresslevel=compresslevel,
        ) as archive:
            archive.write(source_path, arcname=original_filename)
