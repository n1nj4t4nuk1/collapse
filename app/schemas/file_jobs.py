from datetime import datetime

from pydantic import BaseModel, Field

from app.models.job import CompressionAlgorithm, JobStatus


class UploadAcceptedResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    filename: str = Field(description="Original uploaded file name.")
    status: JobStatus = Field(description="Current status of the compression job.")
    algorithm: CompressionAlgorithm = Field(description="Compression algorithm chosen for this job.")
    level: int = Field(description="Compression level (1 = fastest, 5 = maximum).")


class JobStatusResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    filename: str = Field(description="Original uploaded file name.")
    status: JobStatus = Field(description="Current status of the compression job.")
    created_at: datetime = Field(description="UTC timestamp when the job was created.")
    updated_at: datetime = Field(description="UTC timestamp when the job was last updated.")
    archive_name: str = Field(description="Archive file name generated for the job.")
    algorithm: CompressionAlgorithm = Field(description="Compression algorithm chosen for this job.")
    level: int = Field(description="Compression level (1 = fastest, 5 = maximum).")
    error_message: str | None = Field(default=None, description="Compression error details, if any.")


class DeleteResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    deleted: bool = Field(description="Whether at least one stored file was deleted.")
