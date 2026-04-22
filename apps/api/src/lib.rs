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

use config::AppConfig;
use state::AppState;

pub fn build_router(state: AppState, config: &AppConfig) -> Router {
    Router::new()
        .route("/files", get(routes::list_jobs).post(routes::upload_file))
        .route("/files/completed", delete(routes::delete_completed))
        .route("/files/{job_id}/status", get(routes::get_job_status))
        .route("/files/{job_id}/download", get(routes::download_archive))
        .route("/files/{job_id}", delete(routes::delete_job))
        .layer(DefaultBodyLimit::max(config.max_upload_bytes()))
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}

#[cfg(test)]
mod api_tests;
