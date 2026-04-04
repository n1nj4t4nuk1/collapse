"""
File compression service.

Orchestrates compression job execution using the Strategy pattern.
Each job is run in a thread-pool executor to avoid blocking the event loop.
"""

import asyncio

from app.models.job import CompressionAlgorithm, JobStatus
from app.services.compression_strategies import CompressionStrategy, build_strategy_registry
from app.state import job_registry


class CompressionService:
    """
    Coordinates compression job execution by delegating to the appropriate strategy.

    Drives job status transitions in the registry: QUEUED → COMPRESSING → COMPLETED | FAILED.
    """

    def __init__(self, strategies: dict[CompressionAlgorithm, CompressionStrategy] | None = None) -> None:
        """
        Initialise the service with the available strategy map.

        Args:
            strategies: Optional mapping of algorithm → strategy instance.
                        Defaults to the built-in strategy registry if not provided.
        """
        self._strategies = strategies or build_strategy_registry()

    async def compress_job(self, job_id: str) -> None:
        """
        Execute the compression for the job identified by ``job_id``.

        Marks the job as COMPRESSING before starting. Transitions to COMPLETED on
        success, or FAILED (with the exception message) if an error occurs.
        The blocking compression call is offloaded to a thread-pool executor.
        """
        job_registry.update_status(job_id, JobStatus.COMPRESSING)
        job = job_registry.get(job_id)

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
            job_registry.update_status(job_id, JobStatus.FAILED, str(exc))
            return

        job_registry.update_status(job_id, JobStatus.COMPLETED)

    def _get_strategy(self, algorithm: CompressionAlgorithm) -> CompressionStrategy:
        """
        Return the compression strategy for the given algorithm.

        Raises:
            ValueError: If no strategy is registered for the algorithm.
        """
        strategy = self._strategies.get(algorithm)
        if strategy is None:
            raise ValueError(f"Unsupported compression algorithm: {algorithm}")
        return strategy


compression_service = CompressionService()
