import asyncio
import zipfile

import py7zr

from app.models.job import CompressionAlgorithm, JobStatus
from app.state.registry import job_registry

# Map user levels 1-5 to native compression parameters.
_SEVENZ_PRESETS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}
_ZIP_LEVELS: dict[int, int] = {1: 1, 2: 3, 3: 5, 4: 7, 5: 9}


class CompressionService:
    async def compress_job(self, job_id: str) -> None:
        job_registry.update_status(job_id, JobStatus.COMPRESSING)
        job = job_registry.get(job_id)

        if job is None:
            return

        try:
            if job.algorithm is CompressionAlgorithm.SEVENZ:
                await asyncio.to_thread(
                    self._compress_7z,
                    job.original_path,
                    job.compressed_path,
                    job.original_filename,
                    job.level,
                )
            else:
                await asyncio.to_thread(
                    self._compress_zip,
                    job.original_path,
                    job.compressed_path,
                    job.original_filename,
                    job.level,
                )
        except Exception as exc:
            job_registry.update_status(job_id, JobStatus.FAILED, str(exc))
            return

        job_registry.update_status(job_id, JobStatus.COMPLETED)

    def _compress_7z(self, source_path, archive_path, original_filename, level: int) -> None:
        preset = _SEVENZ_PRESETS[level]
        if level == 5:
            preset |= py7zr.PRESET_EXTREME
        filters = [{"id": py7zr.FILTER_LZMA2, "preset": preset}]
        with py7zr.SevenZipFile(archive_path, mode="w", filters=filters) as archive:
            archive.write(source_path, arcname=original_filename)

    def _compress_zip(self, source_path, archive_path, original_filename, level: int) -> None:
        compresslevel = _ZIP_LEVELS[level]
        with zipfile.ZipFile(archive_path, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=compresslevel) as archive:
            archive.write(source_path, arcname=original_filename)


compression_service = CompressionService()
