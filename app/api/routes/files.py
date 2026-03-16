from uuid import uuid4

from fastapi import APIRouter, File, Form, HTTPException, UploadFile, status
from fastapi.responses import FileResponse

from app.models.job import CompressionAlgorithm, CompressionJob, JobStatus
from app.schemas import (
    BulkDeleteCompletedResponse,
    DeleteResponse,
    JobStatusResponse,
    UploadAcceptedResponse,
)
from app.services.compression_queue import compression_queue_service
from app.services.storage import storage_service
from app.state import job_registry

router = APIRouter(prefix="/files", tags=["files"])


def _get_job_or_404(job_id: str) -> CompressionJob:
    job = job_registry.get(job_id)
    if job is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Job not found.")
    return job


def _build_archive_filename(original_filename: str, algorithm: CompressionAlgorithm) -> str:
    ext = "zip" if algorithm is CompressionAlgorithm.ZIP else "7z"
    return f"{original_filename}.{ext}"


def _to_status_response(job: CompressionJob) -> JobStatusResponse:
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
    jobs = sorted(job_registry.list_all(), key=lambda job: job.created_at)
    return [_to_status_response(job) for job in jobs]


@router.post("", response_model=UploadAcceptedResponse, status_code=status.HTTP_202_ACCEPTED)
async def upload_file(
    file: UploadFile = File(...),
    algorithm: CompressionAlgorithm = Form(CompressionAlgorithm.SEVENZ),
    level: int = Form(5, ge=1, le=5),
) -> UploadAcceptedResponse:
    if not file.filename:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="A file name is required.")

    job_id = uuid4().hex
    original_path = storage_service.build_input_path(file.filename)
    compressed_path = storage_service.build_output_path(job_id, algorithm)
    archive_filename = _build_archive_filename(file.filename, algorithm)

    await storage_service.save_upload(file, original_path)

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
    await compression_queue_service.enqueue(job_id)

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
    job = _get_job_or_404(job_id)
    return _to_status_response(job)


@router.get("/{job_id}/download")
def download_archive(job_id: str) -> FileResponse:
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
    completed_jobs = [job for job in job_registry.list_all() if job.status is JobStatus.COMPLETED]

    deleted_files = 0
    for job in completed_jobs:
        original_deleted = storage_service.delete_file(job.original_path)
        compressed_deleted = storage_service.delete_file(job.compressed_path)

        if original_deleted:
            deleted_files += 1
        if compressed_deleted:
            deleted_files += 1

        job_registry.remove(job.job_id)

    return BulkDeleteCompletedResponse(deleted_jobs=len(completed_jobs), deleted_files=deleted_files)


@router.delete("/{job_id}", response_model=DeleteResponse)
def delete_job_files(job_id: str) -> DeleteResponse:
    job = _get_job_or_404(job_id)

    if job.status in {JobStatus.QUEUED, JobStatus.COMPRESSING}:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Cannot delete files while compression is in progress.")

    original_deleted = storage_service.delete_file(job.original_path)
    compressed_deleted = storage_service.delete_file(job.compressed_path)
    job_registry.remove(job_id)

    return DeleteResponse(job_id=job_id, deleted=original_deleted or compressed_deleted)
