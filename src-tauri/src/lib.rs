use std::fs;
use serde::Serialize;

#[derive(Serialize)]
struct FileInfo {
    display_path: String,
    full_path: String,
    entry_type: String,
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_parent_path(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    match p.parent() {
        Some(parent) => Ok(parent.display().to_string()),
        None => Err("You are already in parent folder".into()),
    }
}

#[tauri::command]
fn get_home_dir() -> Result<String, String> {
    std::env::var("HOME").map_err(|_| "Could not find home directory".to_string())
}

#[tauri::command]
fn get_files(path: String) -> Result<Vec<FileInfo>, String> {
    let entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        files.push(FileInfo {
            display_path: p.file_name().unwrap_or_default().to_string_lossy().into(),
            full_path: p.display().to_string(),
            entry_type: if p.is_dir() { "dir" } else { "file" }.into(),
        });
    }
    Ok(files)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_files, get_parent_path, get_home_dir, open_file]) 
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
