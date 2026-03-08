use std::fs;
use serde::Serialize;
use tauri::{menu::{Menu, MenuItem, Submenu}};
use std::path::Path;
use tauri::{Manager, Emitter, WebviewWindowBuilder, WebviewUrl, AppHandle};
use urlencoding;

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
fn create_dir(path: String) -> Result<(), String> {
    fs::create_dir(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_file(path: String) -> Result<(), String> {
    fs::File::create(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }
    if p.is_dir() {
        fs::remove_dir_all(p).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn view_file(path: String, app: AppHandle) {
    let p = Path::new(&path);
    if p.is_file() {
        if let Some(_window) = app.get_webview_window("main") {
            let label = format!("view_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_millis());
            let encoded_path = urlencoding::encode(&path);
            let url = format!("index.html?view={}", encoded_path);
            let empty_menu = Menu::new(&app).unwrap();
            let _ = WebviewWindowBuilder::new(
                &app,
                label,
                WebviewUrl::App(url.into())
            )
            .menu(empty_menu)
            .title(format!("Viewing: {}", path))
            .build();
        }
    }
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
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
        .setup(|app| {
            let create_dir_i = MenuItem::with_id(app, "create-dir", "Create directory", true, Some("CmdOrCtrl+D"))?;
            let create_file_i = MenuItem::with_id(app, "create-file", "Create file", true, Some("CmdOrCtrl+F"))?;
            let view_file_i = MenuItem::with_id(app, "view-file", "View file", true, Some("CmdOrCtrl+O"))?;
            let open_file_i = MenuItem::with_id(app, "open-file", "Open file", true, Some("Enter"))?;
            let new_window_i = MenuItem::with_id(app, "new-window", "New window", true, Some("CmdOrCtrl+N"))?;
            let file_menu = Submenu::with_items(app, "File", true, &[&create_dir_i, &create_file_i, &open_file_i, &view_file_i, &new_window_i])?;
            let delete_file_i = MenuItem::with_id(app, "delete", "Delete", true, Some("Delete"))?;
            let edit_menu = Submenu::with_items(app, "Edit", true, &[&delete_file_i])?;
            let menu = Menu::with_items(app, &[&file_menu, &edit_menu])?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app_handle, event| {
            if event.id() == "create-dir" {
                let _ = app_handle.emit("create-dir", "");
            }
            if event.id() == "create-file" {
                let _ = app_handle.emit("create-file", "");
            }
            if event.id() == "open-file" {
                let _ = app_handle.emit("open-file", "");
            }
            if event.id() == "view-file" {
                let _ = app_handle.emit("view-file", "");
            }
            if event.id() == "new-window" {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let current_pos = window.outer_position().unwrap_or_default();
                    let current_size = window.inner_size().unwrap_or_default();
                    let label = format!("clone_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap().as_millis());

                    let _ = WebviewWindowBuilder::new(
                        app_handle,
                        label,
                        WebviewUrl::App("index.html".into())
                    )
                    .title("File manager")
                    .position((current_pos.x + 20) as f64, (current_pos.y + 20) as f64)
                    .inner_size(current_size.width as f64, current_size.height as f64)
                    .build();
                }
            }
            if event.id() == "delete" {
                let _ = app_handle.emit("delete", "");
            }
        })
        .invoke_handler(tauri::generate_handler![get_files, get_parent_path, get_home_dir, open_file, create_dir, create_file, delete, view_file, read_text_file]) 
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}