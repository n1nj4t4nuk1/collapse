use std::sync::Arc;

use tokio::sync::mpsc;

use collapse_core::compress;

use crate::models::JobStatus;
use crate::registry::InMemoryJobRegistry;

/// Spawn the compression queue worker task.
///
/// The worker consumes job IDs from the channel and processes them
/// sequentially, mirroring the single-worker asyncio queue from the
/// Python implementation.
pub fn start_compression_worker(
    registry: Arc<InMemoryJobRegistry>,
    mut rx: mpsc::UnboundedReceiver<String>,
) {
    tokio::spawn(async move {
        while let Some(job_id) = rx.recv().await {
            process_job(&registry, &job_id).await;
        }
    });
}

async fn process_job(registry: &InMemoryJobRegistry, job_id: &str) {
    registry.update_status(job_id, JobStatus::Compressing, None);

    let job = match registry.get(job_id) {
        Some(job) => job,
        None => return,
    };

    let source = job.original_path.clone();
    let output = job.compressed_path.clone();
    let arcname = job.original_filename.clone();
    let algorithm = job.algorithm;
    let level = job.level;

    let result = tokio::task::spawn_blocking(move || {
        compress(&source, &output, &arcname, algorithm, level)
    })
    .await;

    match result {
        Ok(Ok(())) => {
            registry.update_status(job_id, JobStatus::Completed, None);
        }
        Ok(Err(e)) => {
            registry.update_status(job_id, JobStatus::Failed, Some(e.to_string()));
        }
        Err(e) => {
            registry.update_status(job_id, JobStatus::Failed, Some(e.to_string()));
        }
    }
}
