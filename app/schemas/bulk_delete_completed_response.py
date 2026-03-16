from pydantic import BaseModel, Field


class BulkDeleteCompletedResponse(BaseModel):
    deleted_jobs: int = Field(description="Number of completed jobs removed from the registry.")
    deleted_files: int = Field(description="Number of files deleted from storage.")
