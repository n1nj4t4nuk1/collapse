use std::path::Path;

use super::CompressionError;

pub fn compress_7z(
    _source: &Path,
    _output: &Path,
    _level: u32,
) -> Result<(), CompressionError> {
    todo!("7z compression not yet implemented")
}
