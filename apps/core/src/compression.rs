mod sevenz;
mod zip;

pub use self::sevenz::compress_7z;
pub use self::zip::compress_zip;

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
