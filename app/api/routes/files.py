from uuid import uuid4

from fastapi import APIRouter, BackgroundTasks, File, HTTPException, UploadFile, status
from fastapi.responses import FileResponse

from app.models.job import CompressionJob, JobStatus
from app.schemas.file_jobs import DeleteResponse, JobStatusResponse, UploadAcceptedResponse
from app.services.compression import compression_service
from app.services.storage import storage_service
from app.state.registry import job_registry

router = APIRouter(prefix="/files", tags=["files"])


def _get_job_or_404(job_id: str) -> CompressionJob:
    job = job_registry.get(job_id)
    if job is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Job not found.")
    return job


@router.post("", response_model=UploadAcceptedResponse, status_code=status.HTTP_202_ACCEPTED)
async def upload_file(background_tasks: BackgroundTasks, file: UploadFile = File(...)) -> UploadAcceptedResponse:
    if not file.filename:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="A file name is required.")

    job_id = uuid4().hex
    original_path = storage_service.build_input_path(file.filename)
    compressed_path = storage_service.build_output_path(job_id)

    await storage_service.save_upload(file, original_path)

    job = CompressionJob(
        job_id=job_id,
        original_filename=file.filename,
        original_path=original_path,
        compressed_path=compressed_path,
    )
    job_registry.add(job)
    background_tasks.add_task(compression_service.compress_job, job_id)

    return UploadAcceptedResponse(job_id=job.job_id, filename=job.original_filename, status=job.status)


@router.get("/{job_id}/status", response_model=JobStatusResponse)
def get_job_status(job_id: str) -> JobStatusResponse:
    job = _get_job_or_404(job_id)
    return JobStatusResponse(
        job_id=job.job_id,
        filename=job.original_filename,
        status=job.status,
        created_at=job.created_at,
        updated_at=job.updated_at,
        archive_name=job.compressed_path.name,
        error_message=job.error_message,
    )


@router.get("/{job_id}/download")
def download_archive(job_id: str) -> FileResponse:
    job = _get_job_or_404(job_id)

    if job.status in {JobStatus.PENDING, JobStatus.COMPRESSING}:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Compression is still in progress.")

    if job.status is JobStatus.FAILED:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=job.error_message or "Compression failed.")

    if not job.compressed_path.exists():
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Archive file not found.")

    return FileResponse(path=job.compressed_path, filename=job.compressed_path.name, media_type="application/x-7z-compressed")


@router.delete("/{job_id}", response_model=DeleteResponse)
def delete_job_files(job_id: str) -> DeleteResponse:
    job = _get_job_or_404(job_id)

    if job.status in {JobStatus.PENDING, JobStatus.COMPRESSING}:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Cannot delete files while compression is in progress.")

    original_deleted = storage_service.delete_file(job.original_path)
    compressed_deleted = storage_service.delete_file(job.compressed_path)
    job_registry.remove(job_id)

    return DeleteResponse(job_id=job_id, deleted=original_deleted or compressed_deleted)
