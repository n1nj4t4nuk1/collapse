import asyncio

import py7zr

from app.models.job import JobStatus
from app.state.registry import job_registry


class CompressionService:
    async def compress_job(self, job_id: str) -> None:
        job_registry.update_status(job_id, JobStatus.COMPRESSING)
        job = job_registry.get(job_id)

        if job is None:
            return

        try:
            await asyncio.to_thread(self._compress_file, job.original_path, job.compressed_path, job.original_filename)
        except Exception as exc:
            job_registry.update_status(job_id, JobStatus.FAILED, str(exc))
            return

        job_registry.update_status(job_id, JobStatus.COMPLETED)

    def _compress_file(self, source_path, archive_path, original_filename) -> None:
        filters = [{"id": py7zr.FILTER_LZMA2, "preset": 9 | py7zr.PRESET_EXTREME}]

        with py7zr.SevenZipFile(archive_path, mode="w", filters=filters) as archive:
            archive.write(source_path, arcname=original_filename)


compression_service = CompressionService()
