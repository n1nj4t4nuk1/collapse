import asyncio
from contextlib import suppress

from app.services.compression import compression_service


class CompressionQueueService:
    def __init__(self) -> None:
        self._queue: asyncio.Queue[str] = asyncio.Queue()
        self._worker_task: asyncio.Task | None = None

    async def start(self) -> None:
        if self._worker_task is not None and not self._worker_task.done():
            return
        self._worker_task = asyncio.create_task(self._worker(), name="compression-queue-worker")

    async def stop(self) -> None:
        if self._worker_task is None:
            return

        self._worker_task.cancel()
        with suppress(asyncio.CancelledError):
            await self._worker_task
        self._worker_task = None

    async def enqueue(self, job_id: str) -> None:
        await self._queue.put(job_id)

    async def _worker(self) -> None:
        while True:
            job_id = await self._queue.get()
            try:
                await compression_service.compress_job(job_id)
            finally:
                self._queue.task_done()


compression_queue_service = CompressionQueueService()
