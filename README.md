# Collapse

Collapse is a FastAPI application that accepts a single file upload, stores it locally, compresses it using a configurable algorithm and compression level, exposes the job status, lets clients download the resulting archive, and allows deleting both the original and compressed files.

## Features

- Upload one file per request.
- Store files locally on disk.
- Choose between **7z** (LZMA2) and **zip** (Deflate) compression.
- Choose a compression level from **1** (fastest) to **5** (maximum).
- Track jobs entirely in memory — no database required.
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
python3 main.py
```

Or with uvicorn directly for development with auto-reload:

```bash
uvicorn app.main:app --reload
```

## API Endpoints

### `POST /files`

Upload a file and start a compression job.

**Form fields:**

| Field       | Type    | Default | Description                                      |
|-------------|---------|---------|--------------------------------------------------|
| `file`      | file    | —       | The file to upload (required).                   |
| `algorithm` | string  | `7z`    | Compression algorithm: `7z` or `zip`.            |
| `level`     | integer | `5`     | Compression level: `1` (fastest) to `5` (max).  |

**Compression level mapping:**

| Level | 7z preset        | ZIP compresslevel |
|-------|------------------|-------------------|
| 1     | 1                | 1                 |
| 2     | 3                | 3                 |
| 3     | 5                | 5                 |
| 4     | 7                | 7                 |
| 5     | 9 + EXTREME      | 9                 |

**Response:** `202 Accepted` — returns `job_id`, `filename`, `status`, `algorithm`, and `level`.

---

### `GET /files/{job_id}/status`

Returns the current status of a compression job, including `algorithm`, `level`, and `archive_name`.

---

### `GET /files/{job_id}/download`

Downloads the compressed archive (`.7z` or `.zip`) once the job status is `completed`.

---

### `DELETE /files/{job_id}`

Deletes the original uploaded file and the compressed archive. Cannot be called while compression is in progress.

## Notes

- Job state lives only in memory. Restarting the application clears the registry.
- The service is intended to run as a single-process app because multiple workers would not share the in-memory job registry.
