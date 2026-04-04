"""Response schema for the bulk deletion of completed jobs."""

from pydantic import BaseModel, Field


class BulkDeleteCompletedResponse(BaseModel):
    """Summarises the result of removing all COMPLETED jobs."""

    deleted_jobs: int = Field(description="Number of completed jobs removed from the registry.")
    deleted_files: int = Field(description="Number of files deleted from storage.")
