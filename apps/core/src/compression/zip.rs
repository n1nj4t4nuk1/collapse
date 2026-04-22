use std::path::Path;

use super::CompressionError;

pub fn compress_zip(
    _source: &Path,
    _output: &Path,
    _level: u32,
) -> Result<(), CompressionError> {
    todo!("ZIP compression not yet implemented")
}
