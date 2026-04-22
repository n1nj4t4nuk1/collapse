use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::mpsc;
use tower_http::services::{ServeDir, ServeFile};

use collapse_api::{build_router, config, queue, registry, state, storage};

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8000;

#[derive(Parser)]
#[command(
    name = "collapse-aio",
    about = "Collapse – all-in-one server (API + frontend)."
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

    let strg = Arc::new(storage::FilesystemStorage::new(
        config::input_dir(),
        config::output_dir(),
    ));
    strg.ensure_directories()
        .expect("Failed to create storage directories");

    let reg = Arc::new(registry::InMemoryJobRegistry::new());

    let (tx, rx) = mpsc::unbounded_channel();
    queue::start_compression_worker(reg.clone(), rx);

    let app_state = state::AppState {
        registry: reg,
        storage: strg,
        queue_tx: tx,
    };

    let mut app = build_router(app_state);

    let static_dir = env::var("COLLAPSE_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("static"));

    let serve = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(static_dir.join("index.html")));
    app = app.fallback_service(serve);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid address");

    println!("Collapse AIO listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server error");
}
