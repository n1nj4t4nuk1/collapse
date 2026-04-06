"""Port for compression strategy implementations."""

from abc import abstractmethod
from pathlib import Path

from app.domain.interface import Interface
from app.domain.models.job import CompressionAlgorithm


class CompressionStrategyPort(Interface):
    """Contract for a single compression algorithm."""

    @property
    @abstractmethod
    def algorithm(self) -> CompressionAlgorithm:
        """Return the algorithm identifier this strategy implements."""

    @abstractmethod
    def compress(
        self,
        source_path: Path,
        archive_path: Path,
        original_filename: str,
        level: int,
    ) -> None:
        """Compress the source file and write the result to *archive_path*.

        Args:
            source_path: Absolute path to the original file.
            archive_path: Destination path for the compressed archive.
            original_filename: Name the file will have inside the archive.
            level: Compression level (1 = fastest, 5 = maximum compression).
        """
