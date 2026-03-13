# Collapse

Collapse is a FastAPI application that accepts a single file upload, stores it locally, compresses it into a 7z archive using maximum compression settings, exposes the compression status, lets clients download the archive, and allows deleting both the original and compressed files.

## Features

- Upload one file per request.
- Store files locally on disk.
- Compress files into `.7z` archives using maximum `LZMA2` settings.
- Track jobs entirely in memory.
- Download the generated archive.
- Delete the original file and the archive.

## Requirements

- Python 3.11+

## Installation

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .
```

## Run the API

```bash
uvicorn app.main:app --reload
```

Or run it directly with Python:

```bash
python3 main.py
```

## API Endpoints

- `POST /files`: upload a single file and start compression.
- `GET /files/{job_id}/status`: inspect the current compression status.
- `GET /files/{job_id}/download`: download the `.7z` archive when ready.
- `DELETE /files/{job_id}`: delete the original file and compressed archive.

## Notes

- Job state lives only in memory. Restarting the application clears the registry.
- The service is intended to run as a single-process app because multiple workers would not share the in-memory job registry.
