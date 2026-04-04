"""Response schema for the deletion of a specific job."""

from pydantic import BaseModel, Field


class DeleteResponse(BaseModel):
    """Confirms the deletion of files associated with a job."""

    job_id: str = Field(description="Unique identifier for the compression job.")
    deleted: bool = Field(description="Whether at least one stored file was deleted.")
