"""Application service that orchestrates single-job compression execution."""

import asyncio

from app.domain.models.job import CompressionAlgorithm, JobStatus
from app.domain.ports.compression_strategy import CompressionStrategyPort
from app.domain.ports.job_registry import JobRegistryPort


class CompressionService:
    """Coordinates compression job execution by delegating to the appropriate strategy.

    Drives job status transitions in the registry:
    QUEUED -> COMPRESSING -> COMPLETED | FAILED.
    """

    def __init__(
        self,
        job_registry: JobRegistryPort,
        strategies: dict[CompressionAlgorithm, CompressionStrategyPort],
    ) -> None:
        self._job_registry = job_registry
        self._strategies = strategies

    async def compress_job(self, job_id: str) -> None:
        """Execute the compression for the job identified by *job_id*.

        Marks the job as COMPRESSING before starting.  Transitions to COMPLETED
        on success, or FAILED (with the exception message) if an error occurs.
        The blocking compression call is offloaded to a thread-pool executor.
        """
        self._job_registry.update_status(job_id, JobStatus.COMPRESSING)
        job = self._job_registry.get(job_id)

        if job is None:
            return

        try:
            strategy = self._get_strategy(job.algorithm)
            await asyncio.to_thread(
                strategy.compress,
                job.original_path,
                job.compressed_path,
                job.original_filename,
                job.level,
            )
        except Exception as exc:
            self._job_registry.update_status(job_id, JobStatus.FAILED, str(exc))
            return

        self._job_registry.update_status(job_id, JobStatus.COMPLETED)

    def _get_strategy(self, algorithm: CompressionAlgorithm) -> CompressionStrategyPort:
        """Return the compression strategy for the given algorithm.

        Raises:
            ValueError: If no strategy is registered for the algorithm.
        """
        strategy = self._strategies.get(algorithm)
        if strategy is None:
            raise ValueError(f"Unsupported compression algorithm: {algorithm}")
        return strategy
