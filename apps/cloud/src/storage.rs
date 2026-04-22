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

#[cfg(test)]
mod tests {
    use super::*;

    fn svc(dir: &std::path::Path) -> FilesystemStorage {
        FilesystemStorage::new(dir.join("input"), dir.join("output"))
    }

    #[test]
    fn ensure_directories_creates_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        s.ensure_directories().unwrap();
        assert!(dir.path().join("input").exists());
        assert!(dir.path().join("output").exists());
    }

    #[test]
    fn ensure_directories_idempotent() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        s.ensure_directories().unwrap();
        s.ensure_directories().unwrap();
    }

    #[test]
    fn build_input_path_in_input_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let path = s.build_input_path("document.pdf");
        assert_eq!(path.parent().unwrap(), dir.path().join("input"));
    }

    #[test]
    fn build_input_path_preserves_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let path = s.build_input_path("archive.tar.gz");
        assert_eq!(path.extension().unwrap(), "gz");
    }

    #[test]
    fn build_input_path_unique_per_call() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let p1 = s.build_input_path("file.txt");
        let p2 = s.build_input_path("file.txt");
        assert_ne!(p1, p2);
    }

    #[test]
    fn build_output_path_zip_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let path = s.build_output_path("job1", Algorithm::Zip);
        assert_eq!(path.extension().unwrap(), "zip");
        assert_eq!(path.parent().unwrap(), dir.path().join("output"));
    }

    #[test]
    fn build_output_path_7z_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let path = s.build_output_path("job1", Algorithm::SevenZ);
        assert_eq!(path.extension().unwrap(), "7z");
    }

    #[test]
    fn build_output_path_filename_is_job_id() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let path = s.build_output_path("myjobid", Algorithm::Zip);
        assert_eq!(path.file_stem().unwrap(), "myjobid");
    }

    #[test]
    fn save_file_and_read_back() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        s.ensure_directories().unwrap();
        let dest = dir.path().join("input").join("test.txt");
        s.save_file(b"hello world", &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"hello world");
    }

    #[test]
    fn save_file_creates_dirs_if_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let dest = dir.path().join("input").join("test.txt");
        s.save_file(b"data", &dest).unwrap();
        assert!(dest.exists());
    }

    #[test]
    fn delete_file_returns_true_when_exists() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        let f = dir.path().join("file.txt");
        std::fs::write(&f, "data").unwrap();
        assert!(s.delete_file(&f));
        assert!(!f.exists());
    }

    #[test]
    fn delete_file_returns_false_when_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let s = svc(dir.path());
        assert!(!s.delete_file(&dir.path().join("ghost.txt")));
    }
}
