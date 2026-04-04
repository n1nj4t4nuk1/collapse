"""
Abstract base class for compression strategies.

Defines the interface that all concrete strategy implementations must satisfy,
following the Strategy design pattern.
"""

from abc import ABC, abstractmethod
from pathlib import Path

from app.models.job import CompressionAlgorithm


class CompressionStrategy(ABC):
    """Common interface for all supported compression algorithms."""

    @property
    @abstractmethod
    def algorithm(self) -> CompressionAlgorithm:
        """Return the algorithm identifier this strategy implements."""
        raise NotImplementedError

    @abstractmethod
    def compress(self, source_path: Path, archive_path: Path, original_filename: str, level: int) -> None:
        """
        Compress the source file and write the result to ``archive_path``.

        Args:
            source_path: Absolute path to the original file.
            archive_path: Destination path for the compressed archive.
            original_filename: Name the file will have inside the archive.
            level: Compression level (1 = fastest, 5 = maximum compression).
        """
        raise NotImplementedError
