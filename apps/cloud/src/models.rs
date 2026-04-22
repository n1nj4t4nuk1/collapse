use std::path::PathBuf;

use chrono::{DateTime, Utc};
use collapse_core::Algorithm;
use serde::{Deserialize, Serialize};

/// Lifecycle states of a compression job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Compressing,
    Completed,
    Failed,
}

/// Represents a single file compression job.
#[derive(Debug, Clone)]
pub struct CompressionJob {
    pub job_id: String,
    pub original_filename: String,
    pub archive_filename: String,
    pub original_path: PathBuf,
    pub compressed_path: PathBuf,
    pub algorithm: Algorithm,
    pub level: u32,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CompressionJob {
    pub fn new(
        job_id: String,
        original_filename: String,
        archive_filename: String,
        original_path: PathBuf,
        compressed_path: PathBuf,
        algorithm: Algorithm,
        level: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            job_id,
            original_filename,
            archive_filename,
            original_path,
            compressed_path,
            algorithm,
            level,
            status: JobStatus::Queued,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
