# API Reference

Base URL: `http://localhost:8000`

All endpoints are under the `/files` prefix.

---

## POST /files

Upload a file and start a background compression job.

### Request

`Content-Type: multipart/form-data`

| Field | Type | Default | Description |
|---|---|---|---|
| `file` | file | *(required)* | The file to upload. |
| `algorithm` | string | `7z` | Compression algorithm: `7z` or `zip`. |
| `level` | integer | `5` | Compression level: `1` (fastest) to `5` (maximum). |

### Compression Level Mapping

| Level | 7z preset | ZIP compresslevel |
|---|---|---|
| 1 | 1 | 1 |
| 2 | 3 | 3 |
| 3 | 5 | 5 |
| 4 | 7 | 7 |
| 5 | 9 + EXTREME | 9 |

### Response — `202 Accepted`

```json
{
  "job_id": "a1b2c3d4e5f6...",
  "filename": "report.pdf",
  "archive_name": "report.pdf.7z",
  "status": "queued",
  "algorithm": "7z",
  "level": 5
}
```

### Errors

| Status | Condition |
|---|---|
| 400 | Empty filename |
| 422 | Level out of range or invalid algorithm |

---

## GET /files

List all compression jobs, ordered by creation date (oldest first).

### Response — `200 OK`

```json
[
  {
    "job_id": "...",
    "filename": "report.pdf",
    "status": "completed",
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:05Z",
    "archive_name": "report.pdf.7z",
    "algorithm": "7z",
    "level": 5,
    "error_message": null
  }
]
```

---

## GET /files/{job_id}/status

Get the current status of a specific job.

### Response — `200 OK`

Same schema as a single item in the list above.

### Errors

| Status | Condition |
|---|---|
| 404 | Job not found |

---

## GET /files/{job_id}/download

Download the compressed archive once the job is complete.

### Response — `200 OK`

Binary file download with appropriate `Content-Type`:
- `application/zip` for ZIP archives
- `application/x-7z-compressed` for 7z archives

### Errors

| Status | Condition |
|---|---|
| 404 | Job not found, or archive file missing from disk |
| 409 | Job is still queued/compressing, or compression failed |

---

## DELETE /files/{job_id}

Delete the original file and compressed archive for a specific job, and remove the job from the registry.

### Response — `200 OK`

```json
{
  "job_id": "...",
  "deleted": true
}
```

`deleted` is `true` if at least one file was removed from disk.

### Errors

| Status | Condition |
|---|---|
| 404 | Job not found |
| 409 | Job is still queued or compressing |

---

## DELETE /files/completed

Bulk-delete all completed jobs and their files.

### Response — `200 OK`

```json
{
  "deleted_jobs": 3,
  "deleted_files": 6
}
```

---

## Job Status Lifecycle

```
QUEUED  ──►  COMPRESSING  ──►  COMPLETED
                  │
                  └──►  FAILED
```

| Status | Meaning |
|---|---|
| `queued` | Waiting in the compression queue |
| `compressing` | Compression is running |
| `completed` | Archive is ready for download |
| `failed` | Compression failed (see `error_message`) |
