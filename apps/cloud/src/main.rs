mod config;
mod error;
mod models;
mod queue;
mod registry;
mod routes;
mod schemas;
mod state;
mod storage;

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::routing::{delete, get};
use axum::Router;
use clap::Parser;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use queue::start_compression_worker;
use registry::InMemoryJobRegistry;
use state::AppState;
use storage::FilesystemStorage;

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8000;

#[derive(Parser)]
#[command(
    name = "collapse-cloud",
    about = "Collapse – file compression API server."
)]
struct Cli {
    /// Host address to bind to (overrides COLLAPSE_HOST env var).
    #[arg(long)]
    host: Option<String>,

    /// Port to listen on (overrides COLLAPSE_PORT env var).
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Resolution order: CLI arg > env var > default.
    let host = cli
        .host
        .or_else(|| env::var("COLLAPSE_HOST").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| DEFAULT_HOST.to_string());

    let port = cli
        .port
        .or_else(|| {
            env::var("COLLAPSE_PORT")
                .ok()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(DEFAULT_PORT);

    // Storage
    let storage = Arc::new(FilesystemStorage::new(
        config::input_dir(),
        config::output_dir(),
    ));
    storage
        .ensure_directories()
        .expect("Failed to create storage directories");

    // Job registry
    let registry = Arc::new(InMemoryJobRegistry::new());

    // Compression queue
    let (tx, rx) = mpsc::unbounded_channel();
    start_compression_worker(registry.clone(), rx);

    let state = AppState {
        registry,
        storage,
        queue_tx: tx,
    };

    // Router
    let mut app = Router::new()
        .route("/files", get(routes::list_jobs).post(routes::upload_file))
        .route("/files/completed", delete(routes::delete_completed))
        .route(
            "/files/{job_id}/status",
            get(routes::get_job_status),
        )
        .route(
            "/files/{job_id}/download",
            get(routes::download_archive),
        )
        .route("/files/{job_id}", delete(routes::delete_job))
        .layer(CorsLayer::very_permissive())
        .with_state(state);

    // Serve static frontend if available
    let static_dir = PathBuf::from("static");
    if static_dir.is_dir() {
        let serve = ServeDir::new(&static_dir)
            .not_found_service(ServeFile::new(static_dir.join("index.html")));
        app = app.fallback_service(serve);
    }

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid address");

    println!("Collapse listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server error");
}
