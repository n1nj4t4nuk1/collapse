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

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &[u8] = b"Hello, Collapse! Hello, Collapse! Hello, Collapse! ";

    fn source_file(dir: &std::path::Path) -> std::path::PathBuf {
        let p = dir.join("sample.txt");
        std::fs::write(&p, SAMPLE).unwrap();
        p
    }

    #[test]
    fn creates_valid_zip() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.zip");

        compress_zip(&src, &archive, "sample.txt", 1).unwrap();

        assert!(archive.exists());
        let f = std::fs::File::open(&archive).unwrap();
        assert!(zip::ZipArchive::new(f).is_ok());
    }

    #[test]
    fn zip_contains_original_filename() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.zip");

        compress_zip(&src, &archive, "my_original.txt", 1).unwrap();

        let f = std::fs::File::open(&archive).unwrap();
        let ar = zip::ZipArchive::new(f).unwrap();
        assert!(ar.file_names().any(|n| n == "my_original.txt"));
    }

    #[test]
    fn zip_content_is_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.zip");

        compress_zip(&src, &archive, "sample.txt", 3).unwrap();

        let f = std::fs::File::open(&archive).unwrap();
        let mut ar = zip::ZipArchive::new(f).unwrap();
        let mut entry = ar.by_name("sample.txt").unwrap();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, SAMPLE);
    }

    #[test]
    fn all_levels_produce_valid_zip() {
        for level in 1..=5 {
            let dir = tempfile::TempDir::new().unwrap();
            let src = source_file(dir.path());
            let archive = dir.path().join(format!("out_l{level}.zip"));

            compress_zip(&src, &archive, "sample.txt", level).unwrap();

            let f = std::fs::File::open(&archive).unwrap();
            assert!(zip::ZipArchive::new(f).is_ok(), "level {level} failed");
        }
    }
}
