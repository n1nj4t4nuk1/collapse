use std::path::PathBuf;

use collapse_core::{compress, extract, Algorithm};

#[tauri::command]
fn compress_file(
    file: String,
    output: Option<String>,
    protocol: String,
    level: u32,
) -> Result<String, String> {
    let source = PathBuf::from(&file);

    if !source.exists() {
        return Err(format!("File not found: {file}"));
    }

    let algorithm: Algorithm = protocol.parse().map_err(|e: String| e)?;

    let output_path = match output {
        Some(p) => PathBuf::from(p),
        None => {
            let name = source.file_name().unwrap().to_string_lossy();
            source
                .parent()
                .unwrap()
                .join(format!("{}.{}", name, algorithm.extension()))
        }
    };

    let arcname = source.file_name().unwrap().to_string_lossy().to_string();

    compress(&source, &output_path, &arcname, algorithm, level).map_err(|e| e.to_string())?;

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
fn extract_file(
    archive: String,
    output_dir: Option<String>,
) -> Result<Vec<String>, String> {
    let archive_path = PathBuf::from(&archive);

    if !archive_path.exists() {
        return Err(format!("File not found: {archive}"));
    }

    let output = match output_dir {
        Some(p) => PathBuf::from(p),
        None => archive_path.parent().unwrap().to_path_buf(),
    };

    extract(&archive_path, &output).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![compress_file, extract_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
