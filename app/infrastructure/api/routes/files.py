"""Inbound HTTP adapter — API endpoints for file and compression job management.

Prefix: /files

Available routes:
  GET    /files                   - List all jobs.
  POST   /files                   - Upload a file and enqueue its compression.
  GET    /files/{job_id}/status   - Get the status of a job.
  GET    /files/{job_id}/download - Download the compressed archive.
  DELETE /files/completed         - Delete all completed jobs.
  DELETE /files/{job_id}          - Delete the files for a specific job.
"""

import asyncio
from uuid import uuid4

from fastapi import APIRouter, File, Form, HTTPException, UploadFile, status
from fastapi.responses import FileResponse

import app.container as _di
from app.domain.models.job import CompressionAlgorithm, CompressionJob, JobStatus
from app.infrastructure.api.schemas import (
    BulkDeleteCompletedResponse,
    DeleteResponse,
    JobStatusResponse,
    UploadAcceptedResponse,
)

router = APIRouter(prefix="/files", tags=["files"])


def _get_job_or_404(job_id: str) -> CompressionJob:
    """Return the job or raise HTTP 404 if it does not exist."""
    job = _di.container.job_registry.get(job_id)
    if job is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Job not found.")
    return job


def _build_archive_filename(original_filename: str, algorithm: CompressionAlgorithm) -> str:
    """Build the archive filename by appending the algorithm extension."""
    ext = "zip" if algorithm is CompressionAlgorithm.ZIP else "7z"
    return f"{original_filename}.{ext}"


def _to_status_response(job: CompressionJob) -> JobStatusResponse:
    """Convert a ``CompressionJob`` into its status response schema."""
    return JobStatusResponse(
        job_id=job.job_id,
        filename=job.original_filename,
        status=job.status,
        created_at=job.created_at,
        updated_at=job.updated_at,
        archive_name=job.archive_filename,
        algorithm=job.algorithm,
        level=job.level,
        error_message=job.error_message,
    )


@router.get("", response_model=list[JobStatusResponse])
def list_jobs() -> list[JobStatusResponse]:
    """List all compression jobs ordered by creation date."""
    jobs = sorted(_di.container.job_registry.list_all(), key=lambda job: job.created_at)
    return [_to_status_response(job) for job in jobs]


@router.post("", response_model=UploadAcceptedResponse, status_code=status.HTTP_202_ACCEPTED)
async def upload_file(
    file: UploadFile = File(...),
    algorithm: CompressionAlgorithm = Form(CompressionAlgorithm.SEVENZ),
    level: int = Form(5, ge=1, le=5),
) -> UploadAcceptedResponse:
    """Upload a file and enqueue it for background compression.

    - **file**: File to compress (required).
    - **algorithm**: Compression algorithm (``zip`` or ``7z``). Defaults to ``7z``.
    - **level**: Compression level from 1 (fastest) to 5 (maximum). Defaults to 5.

    Returns 202 Accepted with the details of the created job.
    """
    if not file.filename:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="A file name is required.")

    file_storage = _di.container.file_storage
    job_registry = _di.container.job_registry

    job_id = uuid4().hex
    original_path = file_storage.build_input_path(file.filename)
    compressed_path = file_storage.build_output_path(job_id, algorithm)
    archive_filename = _build_archive_filename(file.filename, algorithm)

    await asyncio.to_thread(file_storage.save_file, file.file, original_path)

    job = CompressionJob(
        job_id=job_id,
        original_filename=file.filename,
        archive_filename=archive_filename,
        original_path=original_path,
        compressed_path=compressed_path,
        algorithm=algorithm,
        level=level,
    )
    job_registry.add(job)
    await _di.container.compression_queue.enqueue(job_id)

    return UploadAcceptedResponse(
        job_id=job.job_id,
        filename=job.original_filename,
        archive_name=job.archive_filename,
        status=job.status,
        algorithm=job.algorithm,
        level=job.level,
    )


@router.get("/{job_id}/status", response_model=JobStatusResponse)
def get_job_status(job_id: str) -> JobStatusResponse:
    """Return the current status of the compression job identified by *job_id*."""
    job = _get_job_or_404(job_id)
    return _to_status_response(job)


@router.get("/{job_id}/download")
def download_archive(job_id: str) -> FileResponse:
    """Download the compressed archive of a completed job.

    Returns 409 Conflict if compression is still in progress or has failed.
    Returns 404 Not Found if the archive no longer exists on disk.
    """
    job = _get_job_or_404(job_id)

    if job.status in {JobStatus.QUEUED, JobStatus.COMPRESSING}:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Compression is still in progress.")

    if job.status is JobStatus.FAILED:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=job.error_message or "Compression failed.")

    if not job.compressed_path.exists():
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Archive file not found.")

    media_type = "application/zip" if job.algorithm is CompressionAlgorithm.ZIP else "application/x-7z-compressed"
    return FileResponse(path=job.compressed_path, filename=job.archive_filename, media_type=media_type)


@router.delete("/completed", response_model=BulkDeleteCompletedResponse)
def delete_completed_jobs() -> BulkDeleteCompletedResponse:
    """Delete all COMPLETED jobs and their associated files.

    Returns a summary with the number of jobs and files removed.
    """
    job_registry = _di.container.job_registry
    file_storage = _di.container.file_storage

    completed_jobs = [job for job in job_registry.list_all() if job.status is JobStatus.COMPLETED]

    deleted_files = 0
    for job in completed_jobs:
        if file_storage.delete_file(job.original_path):
            deleted_files += 1
        if file_storage.delete_file(job.compressed_path):
            deleted_files += 1
        job_registry.remove(job.job_id)

    return BulkDeleteCompletedResponse(deleted_jobs=len(completed_jobs), deleted_files=deleted_files)


@router.delete("/{job_id}", response_model=DeleteResponse)
def delete_job_files(job_id: str) -> DeleteResponse:
    """Delete the files of a specific job and remove it from the registry.

    Returns 409 Conflict if compression is still in progress.
    """
    job = _get_job_or_404(job_id)

    if job.status in {JobStatus.QUEUED, JobStatus.COMPRESSING}:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Cannot delete files while compression is in progress.")

    file_storage = _di.container.file_storage
    original_deleted = file_storage.delete_file(job.original_path)
    compressed_deleted = file_storage.delete_file(job.compressed_path)
    _di.container.job_registry.remove(job_id)

    return DeleteResponse(job_id=job_id, deleted=original_deleted or compressed_deleted)
