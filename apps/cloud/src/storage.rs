use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use collapse_core::Algorithm;
use uuid::Uuid;

/// Stores files on the local filesystem.
pub struct FilesystemStorage {
    input_dir: PathBuf,
    output_dir: PathBuf,
    chunk_size: usize,
}

impl FilesystemStorage {
    pub fn new(input_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            input_dir,
            output_dir,
            chunk_size: 1024 * 1024,
        }
    }

    /// Create the required storage directories if they do not exist.
    pub fn ensure_directories(&self) -> io::Result<()> {
        fs::create_dir_all(&self.input_dir)?;
        fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }

    /// Generate a unique path for storing an uploaded file.
    pub fn build_input_path(&self, filename: &str) -> PathBuf {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        self.input_dir
            .join(format!("{}{}", Uuid::new_v4().simple(), ext))
    }

    /// Generate the output path for a compressed archive.
    pub fn build_output_path(&self, job_id: &str, algorithm: Algorithm) -> PathBuf {
        self.output_dir
            .join(format!("{}.{}", job_id, algorithm.extension()))
    }

    /// Persist raw bytes to the given destination path.
    pub fn save_file(&self, data: &[u8], destination: &Path) -> io::Result<()> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(destination)?;
        for chunk in data.chunks(self.chunk_size) {
            file.write_all(chunk)?;
        }
        Ok(())
    }

    /// Delete the file at `path`. Returns `true` if it existed.
    pub fn delete_file(&self, path: &Path) -> bool {
        fs::remove_file(path).is_ok()
    }
}
