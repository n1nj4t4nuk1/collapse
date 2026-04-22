use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::mpsc;
use tower_http::services::{ServeDir, ServeFile};

use collapse_api::{build_router, config, queue, registry, state, storage};

#[derive(Parser)]
#[command(
    name = "collapse-api",
    about = "Collapse – file compression API server."
)]
struct Cli {
    /// Path to configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Host address to bind to (overrides config file and COLLAPSE_HOST).
    #[arg(long)]
    host: Option<String>,

    /// Port to listen on (overrides config file and COLLAPSE_PORT).
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let cfg = match &cli.config {
        Some(path) => config::AppConfig::from_file(path).expect("Failed to load config"),
        None => config::AppConfig::load_default(),
    };

    let host = cli
        .host
        .or_else(|| env::var("COLLAPSE_HOST").ok().filter(|s| !s.is_empty()))
        .unwrap_or(cfg.server.host.clone());

    let port = cli
        .port
        .or_else(|| {
            env::var("COLLAPSE_PORT")
                .ok()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(cfg.server.port);

    let strg = Arc::new(storage::FilesystemStorage::new(
        cfg.storage.input_dir.clone(),
        cfg.storage.output_dir.clone(),
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

    let mut app = build_router(app_state, &cfg);

    let static_dir = &cfg.static_files.dir;
    if static_dir.is_dir() {
        let serve = ServeDir::new(static_dir)
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
