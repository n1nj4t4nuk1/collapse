"""Composition root — wires ports to their concrete adapters.

All dependency injection happens here.  The rest of the application
accesses dependencies through the module-level ``container`` instance.
"""

from app.application.services.compression_queue import CompressionQueueService
from app.application.services.compression_service import CompressionService
from app.domain.models.job import CompressionAlgorithm
from app.domain.ports.compression_strategy import CompressionStrategyPort
from app.domain.ports.file_storage import FileStoragePort
from app.domain.ports.job_registry import JobRegistryPort
from app.infrastructure.compression.sevenz_strategy import SevenZipCompression
from app.infrastructure.compression.zip_strategy import ZipCompression
from app.infrastructure.config import INPUT_DIR, MAX_UPLOAD_CHUNK_SIZE, OUTPUT_DIR
from app.infrastructure.persistence.in_memory_job_registry import InMemoryJobRegistry
from app.infrastructure.storage.filesystem_storage import FilesystemStorage


class Container:
    """Holds all application dependencies wired together."""

    def __init__(
        self,
        job_registry: JobRegistryPort | None = None,
        file_storage: FileStoragePort | None = None,
        compression_strategies: dict[CompressionAlgorithm, CompressionStrategyPort] | None = None,
    ) -> None:
        self.job_registry: JobRegistryPort = job_registry or InMemoryJobRegistry()

        self.file_storage: FileStoragePort = file_storage or FilesystemStorage(
            input_dir=INPUT_DIR,
            output_dir=OUTPUT_DIR,
            chunk_size=MAX_UPLOAD_CHUNK_SIZE,
        )

        strategies = compression_strategies or {
            CompressionAlgorithm.SEVENZ: SevenZipCompression(),
            CompressionAlgorithm.ZIP: ZipCompression(),
        }

        self.compression_service = CompressionService(
            job_registry=self.job_registry,
            strategies=strategies,
        )

        self.compression_queue = CompressionQueueService(
            process_job=self.compression_service.compress_job,
        )


container = Container()
