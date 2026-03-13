from abc import ABC, abstractmethod
from pathlib import Path

from app.models.job import CompressionAlgorithm


class CompressionStrategy(ABC):
    @property
    @abstractmethod
    def algorithm(self) -> CompressionAlgorithm:
        raise NotImplementedError

    @abstractmethod
    def compress(self, source_path: Path, archive_path: Path, original_filename: str, level: int) -> None:
        raise NotImplementedError
