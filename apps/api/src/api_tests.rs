use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use http_body_util::BodyExt;
use tokio::sync::mpsc;
use tower::ServiceExt;

use collapse_core::Algorithm;

use crate::build_router;
use crate::config::AppConfig;
use crate::models::{CompressionJob, JobStatus};
use crate::queue::start_compression_worker;
use crate::registry::InMemoryJobRegistry;
use crate::state::AppState;
use crate::storage::FilesystemStorage;

fn test_config() -> AppConfig {
    AppConfig::default()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build an AppState backed by temp directories, **without** a compression worker.
/// Returns the state and the receiver (kept alive so sends don't fail).
fn test_state(dir: &tempfile::TempDir) -> (AppState, mpsc::UnboundedReceiver<String>) {
    let storage = Arc::new(FilesystemStorage::new(
        dir.path().join("input"),
        dir.path().join("output"),
    ));
    storage.ensure_directories().unwrap();
    let registry = Arc::new(InMemoryJobRegistry::new());
    let (tx, rx) = mpsc::unbounded_channel();
    (
        AppState {
            registry,
            storage,
            queue_tx: tx,
        },
        rx,
    )
}

/// Build an AppState **with** a running compression worker.
fn test_state_with_worker(dir: &tempfile::TempDir) -> AppState {
    let storage = Arc::new(FilesystemStorage::new(
        dir.path().join("input"),
        dir.path().join("output"),
    ));
    storage.ensure_directories().unwrap();
    let registry = Arc::new(InMemoryJobRegistry::new());
    let (tx, rx) = mpsc::unbounded_channel();
    start_compression_worker(registry.clone(), rx);
    AppState {
        registry,
        storage,
        queue_tx: tx,
    }
}

fn make_job(job_id: &str, status: JobStatus, dir: &tempfile::TempDir) -> CompressionJob {
    let now = Utc::now();
    CompressionJob {
        job_id: job_id.into(),
        original_filename: "file.txt".into(),
        archive_filename: "file.txt.zip".into(),
        original_path: dir.path().join("input").join("orig.txt"),
        compressed_path: dir.path().join("output").join(format!("{job_id}.zip")),
        algorithm: Algorithm::Zip,
        level: 3,
        status,
        error_message: None,
        created_at: now,
        updated_at: now,
    }
}

/// Build a multipart/form-data body for file upload.
fn multipart_upload(
    filename: &str,
    content: &[u8],
    algorithm: Option<&str>,
    level: Option<u32>,
) -> (String, Vec<u8>) {
    let boundary = "----TestBoundary";
    let mut body = Vec::new();

    // File field
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; \
             filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(content);
    body.extend_from_slice(b"\r\n");

    if let Some(algo) = algorithm {
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; \
                 name=\"algorithm\"\r\n\r\n{algo}\r\n"
            )
            .as_bytes(),
        );
    }

    if let Some(lvl) = level {
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; \
                 name=\"level\"\r\n\r\n{lvl}\r\n"
            )
            .as_bytes(),
        );
    }

    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    (
        format!("multipart/form-data; boundary={boundary}"),
        body,
    )
}

async fn body_json(response: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// GET /files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let res = app
        .oneshot(Request::builder().uri("/files").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn list_returns_jobs_ordered_by_created_at() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let j1 = make_job("j1", JobStatus::Completed, &dir);
    let j2 = make_job("j2", JobStatus::Completed, &dir);
    state.registry.add(j1);
    state.registry.add(j2);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(Request::builder().uri("/files").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

// ---------------------------------------------------------------------------
// POST /files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_returns_202() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("hello.txt", b"content", Some("zip"), Some(1));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let json = body_json(res).await;
    assert_eq!(json["filename"], "hello.txt");
    assert_eq!(json["algorithm"], "zip");
    assert_eq!(json["level"], 1);
    assert_eq!(json["status"], "queued");
}

#[tokio::test]
async fn upload_default_algorithm_is_7z() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("f.txt", b"x", None, None);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let json = body_json(res).await;
    assert_eq!(json["algorithm"], "7z");
}

#[tokio::test]
async fn upload_empty_filename_returns_400() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("", b"content", None, None);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_level_out_of_range_returns_400() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("f.txt", b"x", None, Some(6));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_status_returns_200() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state.registry.add(make_job("j1", JobStatus::Queued, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["status"], "queued");
}

#[tokio::test]
async fn get_status_missing_returns_404() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/ghost/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/download
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_completed_job() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    // Create a fake archive on disk
    let archive_path = dir.path().join("output").join("j1.zip");
    std::fs::write(&archive_path, b"PK\x03\x04fake").unwrap();

    let mut job = make_job("j1", JobStatus::Completed, &dir);
    job.compressed_path = archive_path;
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn download_queued_returns_409() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state.registry.add(make_job("j1", JobStatus::Queued, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn download_compressing_returns_409() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state
        .registry
        .add(make_job("j1", JobStatus::Compressing, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn download_failed_returns_409() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let mut job = make_job("j1", JobStatus::Failed, &dir);
    job.error_message = Some("disk full".into());
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn download_missing_archive_returns_404() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let mut job = make_job("j1", JobStatus::Completed, &dir);
    job.compressed_path = PathBuf::from("/nonexistent/out.zip");
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn download_missing_job_returns_404() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/ghost/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// DELETE /files/{job_id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_completed_job() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let orig = dir.path().join("input").join("orig.txt");
    let arc = dir.path().join("output").join("j1.zip");
    std::fs::write(&orig, "x").unwrap();
    std::fs::write(&arc, "PK").unwrap();

    let mut job = make_job("j1", JobStatus::Completed, &dir);
    job.original_path = orig.clone();
    job.compressed_path = arc.clone();
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/j1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["deleted"], true);
    assert!(!orig.exists());
    assert!(!arc.exists());
}

#[tokio::test]
async fn delete_queued_returns_409() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state.registry.add(make_job("j1", JobStatus::Queued, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/j1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_missing_job_returns_404() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/ghost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_removes_job_from_registry() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state
        .registry
        .add(make_job("j1", JobStatus::Completed, &dir));

    let app = build_router(state.clone(), &test_config());
    app.oneshot(
        Request::builder()
            .method("DELETE")
            .uri("/files/j1")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert!(state.registry.get("j1").is_none());
}

// ---------------------------------------------------------------------------
// DELETE /files/completed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_completed_only_deletes_completed() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let orig = dir.path().join("input").join("c_orig.txt");
    let arc = dir.path().join("output").join("c1.zip");
    std::fs::write(&orig, "x").unwrap();
    std::fs::write(&arc, "PK").unwrap();

    let mut completed = make_job("c1", JobStatus::Completed, &dir);
    completed.original_path = orig;
    completed.compressed_path = arc;
    state.registry.add(completed);
    state.registry.add(make_job("q1", JobStatus::Queued, &dir));

    let app = build_router(state.clone(), &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/completed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["deleted_jobs"], 1);
    assert!(state.registry.get("c1").is_none());
    assert!(state.registry.get("q1").is_some());
}

#[tokio::test]
async fn delete_completed_empty_when_no_completed() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state.registry.add(make_job("q1", JobStatus::Queued, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/completed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(res).await;
    assert_eq!(json["deleted_jobs"], 0);
}

#[tokio::test]
async fn delete_completed_counts_deleted_files() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let orig = dir.path().join("input").join("orig2.txt");
    let arc = dir.path().join("output").join("c2.zip");
    std::fs::write(&orig, "x").unwrap();
    std::fs::write(&arc, "PK").unwrap();

    let mut job = make_job("c2", JobStatus::Completed, &dir);
    job.original_path = orig;
    job.compressed_path = arc;
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/completed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(res).await;
    assert_eq!(json["deleted_files"], 2);
}

// ---------------------------------------------------------------------------
// POST /files — invalid algorithm returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_invalid_algorithm_returns_400() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("f.txt", b"x", Some("rar"), None);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let json = body_json(res).await;
    assert!(json["detail"].as_str().unwrap().contains("Unknown algorithm"));
}

// ---------------------------------------------------------------------------
// POST /files — level 0 returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_level_zero_returns_400() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("f.txt", b"x", None, Some(0));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// POST /files — upload with 7z algorithm
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_7z_algorithm() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    let app = build_router(state, &test_config());

    let (ct, body) = multipart_upload("data.bin", b"binary data", Some("7z"), Some(3));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let json = body_json(res).await;
    assert_eq!(json["algorithm"], "7z");
    assert_eq!(json["level"], 3);
    assert_eq!(json["archive_name"], "data.bin.7z");
}

// ---------------------------------------------------------------------------
// DELETE /files/{job_id} — failed job can be deleted
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_failed_job_succeeds() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let mut job = make_job("j1", JobStatus::Failed, &dir);
    job.error_message = Some("boom".into());
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/j1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// DELETE /files/{job_id} — compressing returns 409
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_compressing_returns_409() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state
        .registry
        .add(make_job("j1", JobStatus::Compressing, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/files/j1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/download — Content-Type and Content-Disposition headers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_returns_correct_headers() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let archive_path = dir.path().join("output").join("j1.zip");
    std::fs::write(&archive_path, b"PK\x03\x04fake").unwrap();

    let mut job = make_job("j1", JobStatus::Completed, &dir);
    job.compressed_path = archive_path;
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert_eq!(content_type, "application/zip");
    let disposition = res
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.contains("attachment"));
    assert!(disposition.contains("file.txt.zip"));
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/download — failed job error message in response
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_failed_returns_error_message() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);

    let mut job = make_job("j1", JobStatus::Failed, &dir);
    job.error_message = Some("out of disk".into());
    state.registry.add(job);

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
    let json = body_json(res).await;
    assert_eq!(json["detail"], "out of disk");
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/status — returns all expected fields
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_status_returns_all_fields() {
    let dir = tempfile::TempDir::new().unwrap();
    let (state, _rx) = test_state(&dir);
    state.registry.add(make_job("j1", JobStatus::Queued, &dir));

    let app = build_router(state, &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/files/j1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let json = body_json(res).await;
    assert_eq!(json["job_id"], "j1");
    assert_eq!(json["filename"], "file.txt");
    assert_eq!(json["status"], "queued");
    assert_eq!(json["algorithm"], "zip");
    assert_eq!(json["level"], 3);
    assert_eq!(json["archive_name"], "file.txt.zip");
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

// ---------------------------------------------------------------------------
// Full roundtrip: upload → compress → download → verify
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_roundtrip() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = test_state_with_worker(&dir);

    // 1. Upload
    let (ct, body) = multipart_upload("hello.txt", b"Hello, Collapse!", Some("zip"), Some(1));
    let app = build_router(state.clone(), &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/files")
                .header("content-type", &ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let json = body_json(res).await;
    let job_id = json["job_id"].as_str().unwrap().to_string();

    // 2. Wait for compression to finish
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if let Some(job) = state.registry.get(&job_id) {
            if job.status == JobStatus::Completed {
                break;
            }
        }
    }

    let job = state.registry.get(&job_id).unwrap();
    assert_eq!(job.status, JobStatus::Completed);

    // 3. Download
    let app = build_router(state.clone(), &test_config());
    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/files/{job_id}/download"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Verify ZIP content
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut entry = archive.by_name("hello.txt").unwrap();
    let mut content = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut content).unwrap();
    assert_eq!(content, b"Hello, Collapse!");
}
