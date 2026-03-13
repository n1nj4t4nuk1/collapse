import asyncio

from app.models.job import CompressionAlgorithm, JobStatus
from app.services.compression_strategies import CompressionStrategy, build_strategy_registry
from app.state import job_registry


class CompressionService:
    def __init__(self, strategies: dict[CompressionAlgorithm, CompressionStrategy] | None = None) -> None:
        self._strategies = strategies or build_strategy_registry()

    async def compress_job(self, job_id: str) -> None:
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
        strategy = self._strategies.get(algorithm)
        if strategy is None:
            raise ValueError(f"Unsupported compression algorithm: {algorithm}")
        return strategy


compression_service = CompressionService()
