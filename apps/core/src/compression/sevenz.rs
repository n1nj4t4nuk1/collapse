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
    fn creates_valid_7z() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.7z");

        compress_7z(&src, &archive, "sample.txt", 1).unwrap();
        assert!(archive.exists());
    }

    #[test]
    fn sevenz_contains_original_filename() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.7z");

        compress_7z(&src, &archive, "my_original.txt", 1).unwrap();

        let extract = dir.path().join("extract");
        let file = std::fs::File::open(&archive).unwrap();
        sevenz_rust2::decompress(file, &extract).unwrap();
        assert!(extract.join("my_original.txt").exists());
    }

    #[test]
    fn sevenz_content_is_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = source_file(dir.path());
        let archive = dir.path().join("out.7z");

        compress_7z(&src, &archive, "sample.txt", 1).unwrap();

        let extract = dir.path().join("extract");
        let file = std::fs::File::open(&archive).unwrap();
        sevenz_rust2::decompress(file, &extract).unwrap();
        let content = std::fs::read(extract.join("sample.txt")).unwrap();
        assert_eq!(content, SAMPLE);
    }

    #[test]
    fn all_levels_produce_valid_7z() {
        for level in 1..=5 {
            let dir = tempfile::TempDir::new().unwrap();
            let src = source_file(dir.path());
            let archive = dir.path().join(format!("out_l{level}.7z"));

            compress_7z(&src, &archive, "sample.txt", level).unwrap();
            assert!(archive.exists(), "level {level} failed");
        }
    }
}
