"""Response schema returned immediately when a file upload is accepted."""

from pydantic import BaseModel, Field

from app.domain.models.job import CompressionAlgorithm, JobStatus


class UploadAcceptedResponse(BaseModel):
    """Response returned with HTTP 202 when a compression job is accepted."""

    job_id: str = Field(description="Unique identifier for the compression job.")
    filename: str = Field(description="Original uploaded file name.")
    archive_name: str = Field(description="Archive file name generated for the job.")
    status: JobStatus = Field(description="Current status of the compression job.")
    algorithm: CompressionAlgorithm = Field(description="Compression algorithm chosen for this job.")
    level: int = Field(description="Compression level (1 = fastest, 5 = maximum).")
