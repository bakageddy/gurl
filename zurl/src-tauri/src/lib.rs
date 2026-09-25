use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;

mod hurl;

#[derive(Debug, Serialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<TreeEntry>,
}

#[derive(Debug, Deserialize)]
pub struct SaveFileRequest {
    pub path: String,
    pub content: String,
}

#[command]
fn hurl_version() -> Result<String, String> {
    // Hurl is used as a Rust crate, not an external binary.
    Ok("hurl 8.0.1 (embedded via hurl crate)".to_string())
}

#[command]
fn run_hurl_file(path: String, options: hurl::HurlRunOptions) -> Result<hurl::RunResult, String> {
    hurl::run(hurl::Source::File(PathBuf::from(path)), options)
}

#[command]
fn run_hurl_text(content: String, options: hurl::HurlRunOptions) -> Result<hurl::RunResult, String> {
    hurl::run(hurl::Source::Text(content), options)
}

#[command]
async fn open_file_dialog(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Hurl files", &["hurl", "hur"])
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[command]
async fn open_folder_dialog(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    Ok(picked.map(|p| p.to_string()))
}

#[command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read {path}: {e}"))
}

#[command]
fn write_text_file(request: SaveFileRequest) -> Result<(), String> {
    std::fs::write(&request.path, request.content)
        .map_err(|e| format!("failed to write {}: {e}", request.path))
}

#[command]
fn list_directory_tree(path: String) -> Result<Vec<TreeEntry>, String> {
    fn walk(dir: &std::path::Path) -> Result<Vec<TreeEntry>, String> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(dir).map_err(|e| format!("failed to read directory {}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| format!("failed to read directory entry: {e}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let full_path = entry.path();
            let children = if is_dir { walk(&full_path)? } else { Vec::new() };
            entries.push(TreeEntry {
                name,
                path: full_path.to_string_lossy().to_string(),
                is_dir,
                children,
            });
        }
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        Ok(entries)
    }
    walk(PathBuf::from(path).as_path())
}

#[command]
fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| format!("failed to read directory {path}: {e}"))? {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let full_path = entry.path().to_string_lossy().to_string();
        entries.push(DirEntry {
            name,
            path: full_path,
            is_dir,
        });
    }
    // Folders first, then files, both alphabetically.
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            hurl_version,
            run_hurl_file,
            run_hurl_text,
            open_file_dialog,
            open_folder_dialog,
            read_text_file,
            write_text_file,
            list_directory,
            list_directory_tree
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
