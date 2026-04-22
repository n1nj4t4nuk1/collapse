# API Reference

Base URL: `http://localhost:8000` (configurable via `--host`/`--port` or `COLLAPSE_HOST`/`COLLAPSE_PORT` env vars).

All responses are JSON unless otherwise noted. Errors return `{ "detail": "..." }`.

---

## POST /files

Upload a file and start a compression job.

**Content-Type:** `multipart/form-data`

**Form fields:**

| Field       | Type    | Required | Default | Description |
|-------------|---------|----------|---------|-------------|
| `file`      | file    | yes      | --      | The file to compress |
| `algorithm` | string  | no       | `7z`    | `7z` or `zip` |
| `level`     | integer | no       | `5`     | 1 (fastest) to 5 (max compression) |

**Compression level mapping:**

| Level | 7z LZMA2 preset | ZIP Deflate level |
|-------|-----------------|-------------------|
| 1     | 1               | 1                 |
| 2     | 3               | 3                 |
| 3     | 5               | 5                 |
| 4     | 7               | 7                 |
| 5     | 9               | 9                 |

**Response:** `202 Accepted`

```json
{
  "job_id": "a1b2c3d4...",
  "filename": "report.pdf",
  "archive_name": "report.pdf.7z",
  "status": "queued",
  "algorithm": "7z",
  "level": 5
}
```

**Errors:**

| Status | Cause |
|--------|-------|
| 400 | Missing file, empty filename, or level outside 1--5 |

---

## GET /files

List all compression jobs, ordered by creation time (oldest first).

**Response:** `200 OK`

```json
[
  {
    "job_id": "a1b2c3d4...",
    "filename": "report.pdf",
    "status": "completed",
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:02Z",
    "archive_name": "report.pdf.7z",
    "algorithm": "7z",
    "level": 5,
    "error_message": null
  }
]
```

---

## GET /files/{job_id}/status

Get the current status of a single job.

**Response:** `200 OK` -- same schema as one element of the list above.

**Job status lifecycle:**

```
queued --> compressing --> completed
                      \-> failed
```

**Errors:**

| Status | Cause |
|--------|-------|
| 404 | Job ID not found |

---

## GET /files/{job_id}/download

Download the compressed archive.

**Response:** `200 OK` with binary body.

Headers:
- `Content-Type`: `application/x-7z-compressed` or `application/zip`
- `Content-Disposition`: `attachment; filename="<archive_name>"`

**Errors:**

| Status | Cause |
|--------|-------|
| 404 | Job not found, or archive file missing from disk |
| 409 | Job is still queued, compressing, or failed |

---

## DELETE /files/{job_id}

Delete a single job and its files (original + compressed).

**Response:** `200 OK`

```json
{
  "job_id": "a1b2c3d4...",
  "deleted": true
}
```

`deleted` is `true` if at least one file was removed from disk.

**Errors:**

| Status | Cause |
|--------|-------|
| 404 | Job not found |
| 409 | Job is queued or compressing (cannot delete during processing) |

---

## DELETE /files/completed

Bulk-delete all jobs with status `completed`. Removes both original and compressed files.

**Response:** `200 OK`

```json
{
  "deleted_jobs": 3,
  "deleted_files": 6
}
```

`deleted_files` counts individual files removed (up to 2 per job: original + archive).

---

## CORS

All endpoints have CORS fully open (`CorsLayer::very_permissive()`). This allows the Vue frontend to call the API from any origin during development.
