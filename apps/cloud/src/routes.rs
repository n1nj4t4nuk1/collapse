use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use collapse_core::Algorithm;

use crate::error::AppError;
use crate::models::{CompressionJob, JobStatus};
use crate::schemas::{
    BulkDeleteCompletedResponse, DeleteResponse, JobStatusResponse, UploadAcceptedResponse,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_job_or_404(state: &AppState, job_id: &str) -> Result<CompressionJob, AppError> {
    state
        .registry
        .get(job_id)
        .ok_or_else(|| AppError::NotFound("Job not found.".into()))
}

fn build_archive_filename(original_filename: &str, algorithm: Algorithm) -> String {
    format!("{}.{}", original_filename, algorithm.extension())
}

// ---------------------------------------------------------------------------
// GET /files
// ---------------------------------------------------------------------------

pub async fn list_jobs(State(state): State<AppState>) -> Json<Vec<JobStatusResponse>> {
    let mut jobs = state.registry.list_all();
    jobs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let responses: Vec<JobStatusResponse> = jobs.iter().map(JobStatusResponse::from).collect();
    Json(responses)
}

// ---------------------------------------------------------------------------
// POST /files
// ---------------------------------------------------------------------------

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut algorithm = Algorithm::SevenZ;
    let mut level: u32 = 5;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().map(String::from);
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?
                        .to_vec(),
                );
            }
            "algorithm" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                algorithm = text.parse().unwrap_or(Algorithm::SevenZ);
            }
            "level" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                level = text.parse().unwrap_or(5);
                if !(1..=5).contains(&level) {
                    return Err(AppError::BadRequest(
                        "Level must be between 1 and 5.".into(),
                    ));
                }
            }
            _ => {}
        }
    }

    let filename = filename
        .filter(|f| !f.is_empty())
        .ok_or_else(|| AppError::BadRequest("A file name is required.".into()))?;
    let data =
        file_data.ok_or_else(|| AppError::BadRequest("A file is required.".into()))?;

    let job_id = Uuid::new_v4().simple().to_string();
    let original_path = state.storage.build_input_path(&filename);
    let compressed_path = state.storage.build_output_path(&job_id, algorithm);
    let archive_filename = build_archive_filename(&filename, algorithm);

    // Save uploaded file to disk (blocking I/O offloaded to thread pool).
    let save_path = original_path.clone();
    let storage = state.storage.clone();
    tokio::task::spawn_blocking(move || storage.save_file(&data, &save_path))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let job = CompressionJob::new(
        job_id.clone(),
        filename.clone(),
        archive_filename.clone(),
        original_path,
        compressed_path,
        algorithm,
        level,
    );

    let status = job.status;
    state.registry.add(job);
    state
        .queue_tx
        .send(job_id.clone())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(UploadAcceptedResponse {
            job_id,
            filename,
            archive_name: archive_filename,
            status,
            algorithm,
            level,
        }),
    ))
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/status
// ---------------------------------------------------------------------------

pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, AppError> {
    let job = get_job_or_404(&state, &job_id)?;
    Ok(Json(JobStatusResponse::from(&job)))
}

// ---------------------------------------------------------------------------
// GET /files/{job_id}/download
// ---------------------------------------------------------------------------

pub async fn download_archive(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = get_job_or_404(&state, &job_id)?;

    match job.status {
        JobStatus::Queued | JobStatus::Compressing => {
            return Err(AppError::Conflict(
                "Compression is still in progress.".into(),
            ));
        }
        JobStatus::Failed => {
            return Err(AppError::Conflict(
                job.error_message
                    .unwrap_or_else(|| "Compression failed.".into()),
            ));
        }
        JobStatus::Completed => {}
    }

    if !job.compressed_path.exists() {
        return Err(AppError::NotFound("Archive file not found.".into()));
    }

    let body = tokio::fs::read(&job.compressed_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let headers = [
        (
            header::CONTENT_TYPE,
            job.algorithm.media_type().to_string(),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", job.archive_filename),
        ),
    ];

    Ok((headers, body))
}

// ---------------------------------------------------------------------------
// DELETE /files/completed
// ---------------------------------------------------------------------------

pub async fn delete_completed(
    State(state): State<AppState>,
) -> Json<BulkDeleteCompletedResponse> {
    let jobs = state.registry.list_all();
    let completed: Vec<_> = jobs
        .into_iter()
        .filter(|j| j.status == JobStatus::Completed)
        .collect();

    let mut deleted_files: usize = 0;
    for job in &completed {
        if state.storage.delete_file(&job.original_path) {
            deleted_files += 1;
        }
        if state.storage.delete_file(&job.compressed_path) {
            deleted_files += 1;
        }
        state.registry.remove(&job.job_id);
    }

    Json(BulkDeleteCompletedResponse {
        deleted_jobs: completed.len(),
        deleted_files,
    })
}

// ---------------------------------------------------------------------------
// DELETE /files/{job_id}
// ---------------------------------------------------------------------------

pub async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<DeleteResponse>, AppError> {
    let job = get_job_or_404(&state, &job_id)?;

    if matches!(job.status, JobStatus::Queued | JobStatus::Compressing) {
        return Err(AppError::Conflict(
            "Cannot delete files while compression is in progress.".into(),
        ));
    }

    let original_deleted = state.storage.delete_file(&job.original_path);
    let compressed_deleted = state.storage.delete_file(&job.compressed_path);
    state.registry.remove(&job_id);

    Ok(Json(DeleteResponse {
        job_id,
        deleted: original_deleted || compressed_deleted,
    }))
}
