use std::fs;
use serde::Serialize;
use tauri::{menu::{Menu, MenuItem, Submenu}};
use std::path::Path;
use tauri::{Manager, Emitter, WebviewWindowBuilder, WebviewUrl, AppHandle};
use urlencoding;
use std::process::Command;

#[derive(Serialize)]
struct FileInfo {
    display_path: String,
    full_path: String,
    entry_type: String,
}

static IS_CUT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    files.sort_by_key(|f| (f.entry_type != "dir", f.display_path.to_lowercase()));
    Ok(files)
}

#[tauri::command]
fn paste(dest_dir: String) -> Result<(), String> {
    let wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";
    let output = if wayland {
        Command::new("wl-paste").arg("--type").arg("text/uri-list").output()
    } else {
        Command::new("xclip").arg("-selection").arg("clipboard").arg("-o").arg("-t").arg("text/uri-list").output()
    }.map_err(|e| e.to_string())?;
    let uri = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uri.is_empty() { return Err("Clipboard is empty".into()); }
    let source_path_str = uri.strip_prefix("file://").unwrap_or(&uri);
    let source_path = Path::new(source_path_str);
    let file_name = source_path.file_name().ok_or("Invalid source file name")?;
    let target_path = Path::new(&dest_dir).join(file_name);
    if source_path.is_dir() {
        copy_dir_recursive(source_path, &target_path)?;
    } else {
        fs::copy(source_path, target_path).map_err(|e| e.to_string())?;
    }
    if IS_CUT.load(std::sync::atomic::Ordering::SeqCst) {
        delete(source_path_str.to_string())?;
        IS_CUT.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn copy(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }
    let uri = format!("file://{}", path);
    let wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";
    if wayland {
        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        
        use std::io::Write;
        let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        stdin.write_all(uri.as_bytes()).map_err(|e| e.to_string())?;
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("echo -n '{}' | xclip -selection clipboard -t text/uri-list", uri))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn cut(path: String) -> Result<(), String> {
    IS_CUT.store(true, std::sync::atomic::Ordering::SeqCst);
    let copied = copy(path);
    Ok(copied?)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let create_dir_i = MenuItem::with_id(app, "create-dir", "Create directory", true, Some("CmdOrCtrl+D"))?;
            let create_file_i = MenuItem::with_id(app, "create-file", "Create file", true, Some("CmdOrCtrl+F"))?;
            let view_file_i = MenuItem::with_id(app, "view-file", "View file", true, Some("CmdOrCtrl+O"))?;
            let open_file_i = MenuItem::with_id(app, "open-file", "Open file", true, Some("CmdOrCtrl+Enter"))?;
            let new_window_i = MenuItem::with_id(app, "new-window", "New window", true, Some("CmdOrCtrl+N"))?;
            let file_menu = Submenu::with_items(app, "File", true, &[&create_dir_i, &create_file_i, &open_file_i, &view_file_i, &new_window_i])?;
            
            let copy_file_i = MenuItem::with_id(app, "copy", "Copy", true, Some("CmdOrCtrl+C"))?;
            let paste_file_i = MenuItem::with_id(app, "paste", "Paste", true, Some("CmdOrCtrl+V"))?;
            let cut_file_i = MenuItem::with_id(app, "cut", "Cut", true, Some("CmdOrCtrl+X"))?;
            let delete_file_i = MenuItem::with_id(app, "delete", "Delete", true, Some("Delete"))?;
            let edit_menu = Submenu::with_items(app, "Edit", true, &[&copy_file_i, &paste_file_i, &cut_file_i, &delete_file_i,])?;

            let refresh_i = MenuItem::with_id(app, "refresh", "Refresh", true, Some("CmdOrCtrl+R"))?;
            let show_hidden_i = MenuItem::with_id(app, "toggle-hidden", "Show/hide hidden files", true, Some("CmdOrCtrl+H"))?;
            let view_menu = Submenu::with_items(app, "View", true, &[&refresh_i, &show_hidden_i])?;

            let go_back_i = MenuItem::with_id(app, "go-back", "Go back", true, Some("CmdOrCtrl+Left"))?;
            let go_forward_i = MenuItem::with_id(app, "go-forward", "Go forward", true, Some("CmdOrCtrl+Right"))?;
            let go_up_i = MenuItem::with_id(app, "go-up", "Go up", true, Some("CmdOrCtrl+Up"))?;
            let navigation_menu = Submenu::with_items(app, "Navigation", true, &[&go_back_i, &go_forward_i, &go_up_i])?;
            let menu = Menu::with_items(app, &[&file_menu, &edit_menu, &view_menu, &navigation_menu])?;
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
            if event.id() == "copy" {
                let _ = app_handle.emit("copy", "");
            }
            if event.id() == "paste" {
                let _ = app_handle.emit("paste", "");
            }
            if event.id() == "cut" {
                let _ = app_handle.emit("cut", "");
            }
            if event.id() == "delete" {
                let _ = app_handle.emit("delete", "");
            }
            if event.id() == "refresh" {
                let _ = app_handle.emit("refresh", "");
            }
            if event.id() == "toggle-hidden" {
                let _ = app_handle.emit("toggle-hidden", "");
            }
            if event.id() == "go-back" {
                let _ = app_handle.emit("go-back", "");
            }
            if event.id() == "go-forward" {
                let _ = app_handle.emit("go-forward", "");
            }
            if event.id() == "go-up" {
                let _ = app_handle.emit("go-up", "");
            }
        })
        .invoke_handler(tauri::generate_handler![get_files, get_parent_path, get_home_dir, open_file, create_dir, create_file, delete, view_file, read_text_file, copy, paste, cut]) 
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}