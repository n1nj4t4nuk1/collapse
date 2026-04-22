pub mod config;
pub mod models;
pub mod queue;
pub mod registry;
pub mod state;
pub mod storage;

mod error;
mod routes;
mod schemas;

use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get};
use axum::Router;
use tower_http::cors::CorsLayer;

use state::AppState;

/// Maximum upload size: 500 MB.
const MAX_UPLOAD_SIZE: usize = 500 * 1024 * 1024;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/files", get(routes::list_jobs).post(routes::upload_file))
        .route("/files/completed", delete(routes::delete_completed))
        .route("/files/{job_id}/status", get(routes::get_job_status))
        .route("/files/{job_id}/download", get(routes::download_archive))
        .route("/files/{job_id}", delete(routes::delete_job))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE))
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}

#[cfg(test)]
mod api_tests;
