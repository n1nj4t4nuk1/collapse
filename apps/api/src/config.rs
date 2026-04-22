use std::path::PathBuf;

pub fn storage_dir() -> PathBuf {
    PathBuf::from("storage")
}

pub fn input_dir() -> PathBuf {
    storage_dir().join("input")
}

pub fn output_dir() -> PathBuf {
    storage_dir().join("output")
}
