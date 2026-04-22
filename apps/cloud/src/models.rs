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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_job() -> CompressionJob {
        CompressionJob::new(
            "abc123".into(),
            "file.txt".into(),
            "file.txt.7z".into(),
            PathBuf::from("/tmp/orig.txt"),
            PathBuf::from("/tmp/out.7z"),
            Algorithm::SevenZ,
            3,
        )
    }

    #[test]
    fn job_status_serde_values() {
        assert_eq!(serde_json::to_string(&JobStatus::Queued).unwrap(), "\"queued\"");
        assert_eq!(serde_json::to_string(&JobStatus::Compressing).unwrap(), "\"compressing\"");
        assert_eq!(serde_json::to_string(&JobStatus::Completed).unwrap(), "\"completed\"");
        assert_eq!(serde_json::to_string(&JobStatus::Failed).unwrap(), "\"failed\"");
    }

    #[test]
    fn new_job_defaults() {
        let job = make_job();
        assert_eq!(job.status, JobStatus::Queued);
        assert!(job.error_message.is_none());
    }

    #[test]
    fn touch_updates_updated_at() {
        let mut job = make_job();
        let before = job.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        job.touch();
        assert!(job.updated_at > before);
    }

    #[test]
    fn touch_does_not_change_created_at() {
        let mut job = make_job();
        let created = job.created_at;
        job.touch();
        assert_eq!(job.created_at, created);
    }

    #[test]
    fn timestamps_are_utc() {
        let job = make_job();
        assert!(job.created_at.timezone() == Utc);
        assert!(job.updated_at.timezone() == Utc);
    }
}
