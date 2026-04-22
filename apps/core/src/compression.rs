mod sevenz;
mod zip;

pub use self::sevenz::{compress_7z, extract_7z};
pub use self::zip::{compress_zip, extract_zip};

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported compression algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    #[serde(rename = "7z")]
    SevenZ,
    #[serde(rename = "zip")]
    Zip,
}

impl Algorithm {
    /// File extension for archives produced by this algorithm.
    pub fn extension(&self) -> &str {
        match self {
            Algorithm::SevenZ => "7z",
            Algorithm::Zip => "zip",
        }
    }

    /// MIME type for archives produced by this algorithm.
    pub fn media_type(&self) -> &str {
        match self {
            Algorithm::SevenZ => "application/x-7z-compressed",
            Algorithm::Zip => "application/zip",
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.extension())
    }
}

impl FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "7z" => Ok(Algorithm::SevenZ),
            "zip" => Ok(Algorithm::Zip),
            other => Err(format!("Unknown algorithm: {other}")),
        }
    }
}

impl Algorithm {
    /// Try to detect the algorithm from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "7z" => Some(Algorithm::SevenZ),
            "zip" => Some(Algorithm::Zip),
            _ => None,
        }
    }
}

/// Errors that can occur during compression.
#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Compression failed: {0}")]
    Failed(String),

    #[error("Invalid compression level: {0}. Must be between 1 and 5.")]
    InvalidLevel(u32),
}

/// Compress a file using the given algorithm and level (1–5).
///
/// The file is stored inside the archive under `arcname`.
pub fn compress(
    source: &Path,
    output: &Path,
    arcname: &str,
    algorithm: Algorithm,
    level: u32,
) -> Result<(), CompressionError> {
    if !(1..=5).contains(&level) {
        return Err(CompressionError::InvalidLevel(level));
    }
    match algorithm {
        Algorithm::SevenZ => compress_7z(source, output, arcname, level),
        Algorithm::Zip => compress_zip(source, output, arcname, level),
    }
}

/// Extract an archive into `output_dir`.
///
/// Returns the list of extracted file paths (relative to `output_dir`).
/// The algorithm is detected from the archive file extension.
pub fn extract(archive: &Path, output_dir: &Path) -> Result<Vec<String>, CompressionError> {
    let ext = archive
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let algorithm = Algorithm::from_extension(ext).ok_or_else(|| {
        CompressionError::Failed(format!("Unknown archive extension: .{ext}"))
    })?;

    match algorithm {
        Algorithm::SevenZ => extract_7z(archive, output_dir),
        Algorithm::Zip => extract_zip(archive, output_dir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithm_display() {
        assert_eq!(Algorithm::SevenZ.to_string(), "7z");
        assert_eq!(Algorithm::Zip.to_string(), "zip");
    }

    #[test]
    fn algorithm_from_str() {
        assert_eq!("7z".parse::<Algorithm>().unwrap(), Algorithm::SevenZ);
        assert_eq!("zip".parse::<Algorithm>().unwrap(), Algorithm::Zip);
        assert!("invalid".parse::<Algorithm>().is_err());
    }

    #[test]
    fn algorithm_extension() {
        assert_eq!(Algorithm::SevenZ.extension(), "7z");
        assert_eq!(Algorithm::Zip.extension(), "zip");
    }

    #[test]
    fn algorithm_media_type() {
        assert_eq!(
            Algorithm::SevenZ.media_type(),
            "application/x-7z-compressed"
        );
        assert_eq!(Algorithm::Zip.media_type(), "application/zip");
    }

    #[test]
    fn algorithm_serde_roundtrip() {
        let json = serde_json::to_string(&Algorithm::SevenZ).unwrap();
        assert_eq!(json, "\"7z\"");
        let parsed: Algorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Algorithm::SevenZ);

        let json = serde_json::to_string(&Algorithm::Zip).unwrap();
        assert_eq!(json, "\"zip\"");
        let parsed: Algorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Algorithm::Zip);
    }

    #[test]
    fn compress_invalid_level_zero() {
        let result = compress(Path::new("/x"), Path::new("/y"), "f", Algorithm::Zip, 0);
        assert!(matches!(result, Err(CompressionError::InvalidLevel(0))));
    }

    #[test]
    fn compress_invalid_level_six() {
        let result = compress(Path::new("/x"), Path::new("/y"), "f", Algorithm::Zip, 6);
        assert!(matches!(result, Err(CompressionError::InvalidLevel(6))));
    }
}
