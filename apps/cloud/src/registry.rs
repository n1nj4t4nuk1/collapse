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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use collapse_core::Algorithm;

    fn make_job(id: &str) -> CompressionJob {
        CompressionJob::new(
            id.into(),
            "file.txt".into(),
            "file.txt.7z".into(),
            PathBuf::from("/tmp/orig.txt"),
            PathBuf::from("/tmp/out.7z"),
            Algorithm::SevenZ,
            3,
        )
    }

    #[test]
    fn add_and_get() {
        let reg = InMemoryJobRegistry::new();
        let job = make_job("j1");
        reg.add(job);
        assert!(reg.get("j1").is_some());
        assert_eq!(reg.get("j1").unwrap().job_id, "j1");
    }

    #[test]
    fn get_missing_returns_none() {
        let reg = InMemoryJobRegistry::new();
        assert!(reg.get("ghost").is_none());
    }

    #[test]
    fn list_all_empty() {
        let reg = InMemoryJobRegistry::new();
        assert!(reg.list_all().is_empty());
    }

    #[test]
    fn list_all_returns_all() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        reg.add(make_job("j2"));
        let ids: std::collections::HashSet<String> =
            reg.list_all().into_iter().map(|j| j.job_id).collect();
        assert_eq!(ids, ["j1".to_string(), "j2".to_string()].into());
    }

    #[test]
    fn list_all_returns_copy() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        let mut list = reg.list_all();
        list.clear();
        assert_eq!(reg.list_all().len(), 1);
    }

    #[test]
    fn update_status() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        let updated = reg.update_status("j1", JobStatus::Compressing, None).unwrap();
        assert_eq!(updated.status, JobStatus::Compressing);
    }

    #[test]
    fn update_status_sets_error_message() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        reg.update_status("j1", JobStatus::Failed, Some("boom".into()));
        assert_eq!(reg.get("j1").unwrap().error_message.as_deref(), Some("boom"));
    }

    #[test]
    fn update_status_clears_error_message() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        reg.update_status("j1", JobStatus::Failed, Some("err".into()));
        reg.update_status("j1", JobStatus::Completed, None);
        assert!(reg.get("j1").unwrap().error_message.is_none());
    }

    #[test]
    fn update_status_missing_returns_none() {
        let reg = InMemoryJobRegistry::new();
        assert!(reg.update_status("ghost", JobStatus::Completed, None).is_none());
    }

    #[test]
    fn remove_existing() {
        let reg = InMemoryJobRegistry::new();
        reg.add(make_job("j1"));
        let removed = reg.remove("j1");
        assert!(removed.is_some());
        assert!(reg.get("j1").is_none());
    }

    #[test]
    fn remove_missing_returns_none() {
        let reg = InMemoryJobRegistry::new();
        assert!(reg.remove("ghost").is_none());
    }
}
