from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path


class JobStatus(str, Enum):
    PENDING = "pending"
    COMPRESSING = "compressing"
    COMPLETED = "completed"
    FAILED = "failed"


class CompressionAlgorithm(str, Enum):
    ZIP = "zip"
    SEVENZ = "7z"


@dataclass(slots=True)
class CompressionJob:
    job_id: str
    original_filename: str
    archive_filename: str
    original_path: Path
    compressed_path: Path
    algorithm: CompressionAlgorithm
    level: int
    status: JobStatus = JobStatus.PENDING
    error_message: str | None = None
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def touch(self) -> None:
        self.updated_at = datetime.now(timezone.utc)
