"""Domain ports (interfaces) package."""

from app.domain.ports.compression_strategy import CompressionStrategyPort
from app.domain.ports.file_storage import FileStoragePort
from app.domain.ports.job_registry import JobRegistryPort

__all__ = ["CompressionStrategyPort", "FileStoragePort", "JobRegistryPort"]
