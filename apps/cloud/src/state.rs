use std::sync::Arc;

use tokio::sync::mpsc;

use crate::registry::InMemoryJobRegistry;
use crate::storage::FilesystemStorage;

/// Shared application state, passed to every route handler via Axum's
/// `State` extractor. Equivalent to the Python `Container`.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<InMemoryJobRegistry>,
    pub storage: Arc<FilesystemStorage>,
    pub queue_tx: mpsc::UnboundedSender<String>,
}
