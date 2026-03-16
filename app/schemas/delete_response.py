from pydantic import BaseModel, Field


class DeleteResponse(BaseModel):
    job_id: str = Field(description="Unique identifier for the compression job.")
    deleted: bool = Field(description="Whether at least one stored file was deleted.")
