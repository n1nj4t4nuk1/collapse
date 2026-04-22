use chrono::{DateTime, Utc};
use collapse_core::Algorithm;
use serde::Serialize;

use crate::models::{CompressionJob, JobStatus};

#[derive(Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub filename: String,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archive_name: String,
    pub algorithm: Algorithm,
    pub level: u32,
    pub error_message: Option<String>,
}

impl From<&CompressionJob> for JobStatusResponse {
    fn from(job: &CompressionJob) -> Self {
        Self {
            job_id: job.job_id.clone(),
            filename: job.original_filename.clone(),
            status: job.status,
            created_at: job.created_at,
            updated_at: job.updated_at,
            archive_name: job.archive_filename.clone(),
            algorithm: job.algorithm,
            level: job.level,
            error_message: job.error_message.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct UploadAcceptedResponse {
    pub job_id: String,
    pub filename: String,
    pub archive_name: String,
    pub status: JobStatus,
    pub algorithm: Algorithm,
    pub level: u32,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub job_id: String,
    pub deleted: bool,
}

#[derive(Serialize)]
pub struct BulkDeleteCompletedResponse {
    pub deleted_jobs: usize,
    pub deleted_files: usize,
}
