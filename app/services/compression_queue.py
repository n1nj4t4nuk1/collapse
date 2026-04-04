"""
Async compression job queue.

Decouples HTTP request handling from compression processing.
Jobs are consumed sequentially by a single async worker task.
"""

import asyncio
from contextlib import suppress

from app.services.compression import compression_service


class CompressionQueueService:
    """
    Manages a FIFO queue of compression jobs and the worker that processes them.

    The worker runs as an independent asyncio task and consumes jobs one by one
    until it is stopped.
    """

    def __init__(self) -> None:
        self._queue: asyncio.Queue[str] = asyncio.Queue()
        self._worker_task: asyncio.Task | None = None

    async def start(self) -> None:
        """
        Start the queue worker if it is not already running.

        Safe to call multiple times; a duplicate task is never created.
        """
        if self._worker_task is not None and not self._worker_task.done():
            return
        self._worker_task = asyncio.create_task(self._worker(), name="compression-queue-worker")

    async def stop(self) -> None:
        """
        Cancel the worker and wait for it to finish gracefully.

        ``CancelledError`` is suppressed so that application shutdown remains clean.
        """
        if self._worker_task is None:
            return

        self._worker_task.cancel()
        with suppress(asyncio.CancelledError):
            await self._worker_task
        self._worker_task = None

    async def enqueue(self, job_id: str) -> None:
        """Add a ``job_id`` to the end of the queue for processing."""
        await self._queue.put(job_id)

    async def _worker(self) -> None:
        """
        Main worker loop.

        Waits for jobs in the queue and processes them by calling
        ``compression_service.compress_job``. Each queue item is marked done
        regardless of the outcome.
        """
        while True:
            job_id = await self._queue.get()
            try:
                await compression_service.compress_job(job_id)
            finally:
                self._queue.task_done()


compression_queue_service = CompressionQueueService()
