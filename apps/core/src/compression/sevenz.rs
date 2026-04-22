use std::fs;
use std::path::Path;

use sevenz_rust2::{SevenZArchiveEntry, SevenZWriter};

use super::CompressionError;

/// API level (1–5) → py7zr-equivalent preset (1–9).
const SEVENZ_PRESETS: [u32; 5] = [1, 3, 5, 7, 9];

pub fn compress_7z(
    source: &Path,
    output: &Path,
    arcname: &str,
    level: u32,
) -> Result<(), CompressionError> {
    let _preset = SEVENZ_PRESETS[(level - 1) as usize];

    let content = fs::read(source)?;

    let mut writer =
        SevenZWriter::create(output).map_err(|e| CompressionError::Failed(e.to_string()))?;

    let mut entry = SevenZArchiveEntry::default();
    entry.name = arcname.to_string();

    writer
        .push_archive_entry(entry, Some(content.as_slice()))
        .map_err(|e| CompressionError::Failed(e.to_string()))?;

    writer
        .finish()
        .map_err(|e| CompressionError::Failed(e.to_string()))?;

    Ok(())
}
