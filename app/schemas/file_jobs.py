from datetime import datetime

from pydantic import BaseModel, Field

from app.models.job import JobStatus


class UploadAcceptedResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    filename: str = Field(description="Original uploaded file name.")
    status: JobStatus = Field(description="Current status of the compression job.")


class JobStatusResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    filename: str = Field(description="Original uploaded file name.")
    status: JobStatus = Field(description="Current status of the compression job.")
    created_at: datetime = Field(description="UTC timestamp when the job was created.")
    updated_at: datetime = Field(description="UTC timestamp when the job was last updated.")
    archive_name: str = Field(description="Archive file name generated for the job.")
    error_message: str | None = Field(default=None, description="Compression error details, if any.")


class DeleteResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    deleted: bool = Field(description="Whether at least one stored file was deleted.")
