mod sevenz;
mod zip;

pub use self::sevenz::compress_7z;
pub use self::zip::compress_zip;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    SevenZ,
    Zip,
}

#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compression failed: {0}")]
    Failed(String),
}

/// Compress a file using the given algorithm and level (1-5).
pub fn compress(
    source: &Path,
    output: &Path,
    algorithm: Algorithm,
    level: u32,
) -> Result<(), CompressionError> {
    match algorithm {
        Algorithm::SevenZ => compress_7z(source, output, level),
        Algorithm::Zip => compress_zip(source, output, level),
    }
}
