from contextlib import asynccontextmanager

from fastapi import FastAPI
import uvicorn

from app.api.routes.files import router as files_router
from app.services.storage import storage_service


@asynccontextmanager
async def lifespan(_: FastAPI):
    storage_service.ensure_directories()
    yield


app = FastAPI(
    title="Collapse",
    description="Upload a single file, compress it to 7z, check status, download the archive, and delete stored files.",
    version="0.1.0",
    lifespan=lifespan,
)
app.include_router(files_router)


def run() -> None:
    uvicorn.run("app.main:app", host="127.0.0.1", port=8000, reload=False)


if __name__ == "__main__":
    run()
