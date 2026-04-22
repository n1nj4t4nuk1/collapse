use std::path::{Path, PathBuf};

use serde::Deserialize;

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8000;
const DEFAULT_MAX_UPLOAD_MB: usize = 500;
const DEFAULT_STATIC_DIR: &str = "static";

/// Top-level configuration, mirrors `collapse.toml`.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub upload: UploadConfig,
    #[serde(rename = "static")]
    pub static_files: StaticConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct StorageConfig {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct UploadConfig {
    pub max_size_mb: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct StaticConfig {
    pub dir: PathBuf,
}

// -- Defaults -----------------------------------------------------------------

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            upload: UploadConfig::default(),
            static_files: StaticConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from("storage/input"),
            output_dir: PathBuf::from("storage/output"),
        }
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_size_mb: DEFAULT_MAX_UPLOAD_MB,
        }
    }
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from(DEFAULT_STATIC_DIR),
        }
    }
}

// -- Loading ------------------------------------------------------------------

impl AppConfig {
    /// Load config from a TOML file. Missing fields use defaults.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        toml::from_str(&content).map_err(|e| format!("Invalid config {}: {e}", path.display()))
    }

    /// Load config from `collapse.toml` in the working directory if it exists,
    /// otherwise return defaults.
    pub fn load_default() -> Self {
        let path = PathBuf::from("collapse.toml");
        if path.exists() {
            match Self::from_file(&path) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("warning: {e}, using defaults");
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Max upload size in bytes.
    pub fn max_upload_bytes(&self) -> usize {
        self.upload.max_size_mb * 1024 * 1024
    }
}
