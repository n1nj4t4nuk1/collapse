"""Port for file storage operations."""

from abc import abstractmethod
from pathlib import Path
from typing import BinaryIO

from app.domain.interface import Interface
from app.domain.models.job import CompressionAlgorithm


class FileStoragePort(Interface):
    """Contract for all file I/O operations required by the service."""

    @abstractmethod
    def ensure_directories(self) -> None:
        """Create the required storage directories if they do not exist."""

    @abstractmethod
    def build_input_path(self, filename: str) -> Path:
        """Generate a unique path for storing an uploaded file."""

    @abstractmethod
    def build_output_path(self, job_id: str, algorithm: CompressionAlgorithm) -> Path:
        """Generate the output path for a compressed archive."""

    @abstractmethod
    def save_file(self, source: BinaryIO, destination: Path) -> None:
        """Persist a file from *source* to *destination*."""

    @abstractmethod
    def delete_file(self, path: Path) -> bool:
        """Delete the file at *path*. Return ``True`` if it existed."""
