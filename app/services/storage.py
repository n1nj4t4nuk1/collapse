from pathlib import Path
from uuid import uuid4

from fastapi import UploadFile

from app.core.config import INPUT_DIR, MAX_UPLOAD_CHUNK_SIZE, OUTPUT_DIR


class StorageService:
    def ensure_directories(self) -> None:
        INPUT_DIR.mkdir(parents=True, exist_ok=True)
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    def build_input_path(self, filename: str) -> Path:
        suffix = Path(filename).suffix
        return INPUT_DIR / f"{uuid4().hex}{suffix}"

    def build_output_path(self, job_id: str) -> Path:
        return OUTPUT_DIR / f"{job_id}.7z"

    async def save_upload(self, upload: UploadFile, destination: Path) -> None:
        self.ensure_directories()

        with destination.open("wb") as target:
            while chunk := await upload.read(MAX_UPLOAD_CHUNK_SIZE):
                target.write(chunk)

        await upload.close()

    def delete_file(self, path: Path) -> bool:
        if not path.exists():
            return False

        path.unlink()
        return True


storage_service = StorageService()
