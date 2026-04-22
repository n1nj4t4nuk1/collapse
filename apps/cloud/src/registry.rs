use std::collections::HashMap;
use std::sync::Mutex;

use crate::models::{CompressionJob, JobStatus};

/// In-memory job registry backed by a `HashMap` and protected by a `Mutex`.
///
/// Thread-safe: the `Mutex` guards all reads and writes. Data is **not**
/// persisted across application restarts.
pub struct InMemoryJobRegistry {
    jobs: Mutex<HashMap<String, CompressionJob>>,
}

impl InMemoryJobRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, job: CompressionJob) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.insert(job.job_id.clone(), job);
    }

    pub fn get(&self, job_id: &str) -> Option<CompressionJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).cloned()
    }

    pub fn list_all(&self) -> Vec<CompressionJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.values().cloned().collect()
    }

    pub fn update_status(
        &self,
        job_id: &str,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Option<CompressionJob> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = status;
            job.error_message = error_message;
            job.touch();
            Some(job.clone())
        } else {
            None
        }
    }

    pub fn remove(&self, job_id: &str) -> Option<CompressionJob> {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.remove(job_id)
    }
}
