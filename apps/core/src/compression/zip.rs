use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::CompressionError;

/// API level (1–5) → Deflate compresslevel (1–9).
const ZIP_LEVELS: [i64; 5] = [1, 3, 5, 7, 9];

pub fn compress_zip(
    source: &Path,
    output: &Path,
    arcname: &str,
    level: u32,
) -> Result<(), CompressionError> {
    let compress_level = ZIP_LEVELS[(level - 1) as usize];

    let output_file = File::create(output)?;
    let mut writer = ZipWriter::new(output_file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(compress_level));

    writer
        .start_file(arcname, options)
        .map_err(|e| CompressionError::Failed(e.to_string()))?;

    let mut source_file = File::open(source)?;
    let mut buffer = Vec::new();
    source_file.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;

    writer
        .finish()
        .map_err(|e| CompressionError::Failed(e.to_string()))?;

    Ok(())
}
