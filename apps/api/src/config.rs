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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn defaults_are_sensible() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 8000);
        assert_eq!(cfg.storage.input_dir, PathBuf::from("storage/input"));
        assert_eq!(cfg.storage.output_dir, PathBuf::from("storage/output"));
        assert_eq!(cfg.upload.max_size_mb, 500);
        assert_eq!(cfg.static_files.dir, PathBuf::from("static"));
    }

    #[test]
    fn max_upload_bytes_conversion() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.max_upload_bytes(), 500 * 1024 * 1024);

        let mut cfg2 = AppConfig::default();
        cfg2.upload.max_size_mb = 1;
        assert_eq!(cfg2.max_upload_bytes(), 1024 * 1024);
    }

    #[test]
    fn from_file_full_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"
[server]
host = "127.0.0.1"
port = 3000

[storage]
input_dir = "/tmp/in"
output_dir = "/tmp/out"

[upload]
max_size_mb = 100

[static]
dir = "public"
"#).unwrap();

        let cfg = AppConfig::from_file(&path).unwrap();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 3000);
        assert_eq!(cfg.storage.input_dir, PathBuf::from("/tmp/in"));
        assert_eq!(cfg.storage.output_dir, PathBuf::from("/tmp/out"));
        assert_eq!(cfg.upload.max_size_mb, 100);
        assert_eq!(cfg.static_files.dir, PathBuf::from("public"));
    }

    #[test]
    fn from_file_partial_config_uses_defaults() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("partial.toml");
        std::fs::write(&path, "[server]\nport = 9090\n").unwrap();

        let cfg = AppConfig::from_file(&path).unwrap();
        assert_eq!(cfg.server.port, 9090);
        // Everything else should be default
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.upload.max_size_mb, 500);
    }

    #[test]
    fn from_file_empty_config_uses_all_defaults() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("empty.toml");
        std::fs::write(&path, "").unwrap();

        let cfg = AppConfig::from_file(&path).unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 8000);
    }

    #[test]
    fn from_file_not_found_returns_error() {
        let result = AppConfig::from_file(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot read"));
    }

    #[test]
    fn from_file_invalid_toml_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "[server\nbroken").unwrap();

        let result = AppConfig::from_file(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid config"));
    }
}
