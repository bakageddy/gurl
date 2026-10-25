use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, Debouncer};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

mod error;

use error::{Error, Result};
use gurl_core::{edit, emit, import, model, parse, query, runs, workspace};
use workspace::{EffectiveHeader, EffectiveVar, EnvVar, Node};

#[derive(Default)]
struct AppState {
    root: Mutex<Option<PathBuf>>,
    watcher: Mutex<Option<Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>>>,
    runs: runs::RunStore,
}

impl AppState {
    fn root(&self) -> Result<PathBuf> {
        self.root.lock().unwrap().clone().ok_or(Error::NoWorkspace)
    }
}

fn data_dir(app: &AppHandle) -> Result<PathBuf> {
    Ok(app.path().app_data_dir()?)
}

// ------------------------------------------------------------------- hurl

#[command]
fn hurl_version() -> String {
    "8.0.1".to_string()
}

#[command]
fn parse_hurl(text: String) -> model::ParseOutcome {
    parse::parse(&text)
}

#[command]
fn emit_hurl(model: model::HurlDoc) -> String {
    emit::emit(&model)
}

/// hurlfmt's formatting of one document; the error is the parse error with its position.
#[command]
fn format_hurl(text: String) -> Result<String> {
    gurl_core::cli::fmt::format_text(&text).map_err(Error::Format)
}

// -------------------------------------------------------------- workspace

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceInfo {
    root: String,
    max_depth: usize,
    nodes: Vec<Node>,
}

fn watch(app: &AppHandle, state: &AppState, root: &Path) {
    let handle = app.clone();
    let debouncer = new_debouncer(Duration::from_millis(200), move |res: Result<Vec<_>, _>| {
        let Ok(events) = res else { return };
        let mut dirs: Vec<String> = events
            .iter()
            .filter_map(|e: &notify_debouncer_mini::DebouncedEvent| {
                e.path.parent().map(|p| p.to_string_lossy().to_string())
            })
            .collect();
        dirs.sort();
        dirs.dedup();
        let _ = handle.emit("workspace-changed", dirs);
    });
    if let Ok(mut d) = debouncer {
        let _ = d.watcher().watch(root, RecursiveMode::Recursive);
        *state.watcher.lock().unwrap() = Some(d);
    }
}

#[command]
fn open_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<WorkspaceInfo> {
    let root = std::fs::canonicalize(&path).map_err(Error::io("cannot open", Path::new(&path)))?;
    if !root.is_dir() {
        return Err(Error::NotADirectory(root));
    }
    let nodes = workspace::list_dir(&root, &root)?;
    *state.root.lock().unwrap() = Some(root.clone());
    watch(&app, &state, &root);
    Ok(WorkspaceInfo {
        root: root.to_string_lossy().to_string(),
        max_depth: workspace::MAX_DEPTH,
        nodes,
    })
}

#[command]
fn close_workspace(state: State<'_, AppState>) {
    *state.root.lock().unwrap() = None;
    *state.watcher.lock().unwrap() = None;
}

#[command]
fn list_dir(state: State<'_, AppState>, path: String) -> Result<Vec<Node>> {
    Ok(workspace::list_dir(&state.root()?, Path::new(&path))?)
}

#[command]
fn create_file(
    state: State<'_, AppState>,
    dir: String,
    name: String,
    content: Option<String>,
) -> Result<String> {
    let p = workspace::create_file(
        &state.root()?,
        Path::new(&dir),
        &name,
        content.as_deref().unwrap_or(""),
    )?;
    Ok(p.to_string_lossy().to_string())
}

#[command]
fn create_dir(state: State<'_, AppState>, parent: String, name: String) -> Result<String> {
    let p = workspace::create_dir(&state.root()?, Path::new(&parent), &name)?;
    Ok(p.to_string_lossy().to_string())
}

#[command]
fn rename_path(state: State<'_, AppState>, path: String, new_name: String) -> Result<String> {
    let p = workspace::rename(&state.root()?, Path::new(&path), &new_name)?;
    Ok(p.to_string_lossy().to_string())
}

#[command]
fn delete_path(state: State<'_, AppState>, path: String) -> Result<()> {
    Ok(workspace::delete(&state.root()?, Path::new(&path))?)
}

#[command]
fn duplicate_path(state: State<'_, AppState>, path: String) -> Result<String> {
    let p = workspace::duplicate(&state.root()?, Path::new(&path))?;
    Ok(p.to_string_lossy().to_string())
}

#[command]
fn get_dir_vars(state: State<'_, AppState>, dir: String) -> Result<Vec<EnvVar>> {
    let root = state.root()?;
    let dir = workspace::contain(&root, Path::new(&dir))?;
    Ok(workspace::read_env(&dir))
}

#[command]
fn set_dir_vars(state: State<'_, AppState>, dir: String, vars: Vec<EnvVar>) -> Result<()> {
    Ok(workspace::write_env(
        &state.root()?,
        Path::new(&dir),
        &vars,
    )?)
}

#[command]
fn get_env_text(state: State<'_, AppState>, dir: String) -> Result<String> {
    Ok(workspace::read_env_text(&state.root()?, Path::new(&dir))?)
}

#[command]
fn set_env_text(state: State<'_, AppState>, dir: String, text: String) -> Result<()> {
    Ok(workspace::write_env_text(
        &state.root()?,
        Path::new(&dir),
        &text,
    )?)
}

#[command]
fn get_effective_vars(state: State<'_, AppState>, dir: String) -> Result<Vec<EffectiveVar>> {
    Ok(workspace::effective_vars(&state.root()?, Path::new(&dir))?)
}

#[command]
fn get_headers_text(state: State<'_, AppState>, dir: String) -> Result<String> {
    Ok(workspace::read_headers_text(&state.root()?, Path::new(&dir))?)
}

#[command]
fn set_headers_text(state: State<'_, AppState>, dir: String, text: String) -> Result<()> {
    Ok(workspace::write_headers_text(
        &state.root()?,
        Path::new(&dir),
        &text,
    )?)
}

#[command]
fn get_effective_headers(
    state: State<'_, AppState>,
    dir: String,
) -> Result<Vec<EffectiveHeader>> {
    Ok(workspace::effective_headers(&state.root()?, Path::new(&dir))?)
}

// Plain file access. Files inside the workspace are contained to it; files opened via the
// "open file" dialog live outside and are allowed only for `.hurl` extensions.

fn check_file_access(state: &AppState, path: &Path) -> Result<()> {
    if let Some(root) = state.root.lock().unwrap().as_ref() {
        if workspace::contain(root, path).is_ok() {
            return Ok(());
        }
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("hurl") | Some("hur") => Ok(()),
        _ => Err(Error::OutsideAccess),
    }
}

#[command]
fn read_text_file(state: State<'_, AppState>, path: String) -> Result<String> {
    check_file_access(&state, Path::new(&path))?;
    std::fs::read_to_string(&path).map_err(Error::io("failed to read", Path::new(&path)))
}

#[command]
fn write_text_file(state: State<'_, AppState>, path: String, content: String) -> Result<()> {
    check_file_access(&state, Path::new(&path))?;
    std::fs::write(&path, content).map_err(Error::io("failed to write", Path::new(&path)))
}

#[command]
async fn open_file_dialog(app: AppHandle) -> Result<Option<String>> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Hurl files", &["hurl", "hur"])
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[command]
async fn open_folder_dialog(app: AppHandle) -> Result<Option<String>> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string()))
}

/// Lets the user choose a data file for a multipart part / body. The file must live in the
/// scenario's directory (or below); the returned path is relative to the `.hurl` file.
#[command]
async fn pick_data_file(app: AppHandle, hurl_path: String) -> Result<Option<String>> {
    let hurl = PathBuf::from(&hurl_path);
    let dir = hurl.parent().map(Path::to_path_buf);
    let mut dialog = app.dialog().file();
    if let Some(d) = dir {
        dialog = dialog.set_directory(d);
    }
    let Some(picked) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    let picked = picked
        .into_path()
        .map_err(|e| Error::NotLocalPath(Box::new(e)))?;
    Ok(Some(workspace::data_file_relative(&hurl, &picked)?))
}

#[command]
fn check_data_file(hurl_path: String, reference: String) -> workspace::DataFileStatus {
    workspace::check_data_file(Path::new(&hurl_path), &reference)
}

// ------------------------------------------------------------------- runs

#[command]
async fn run_scope(
    app: AppHandle,
    request: runs::ScopeRequest,
    on_event: Channel<runs::RunEvent>,
) -> Result<runs::RunSummary> {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let root = state.root.lock().unwrap().clone();
        let summary = state.runs.execute(root, request, &|e| {
            let _ = on_event.send(e);
        })?;
        if let (Ok(dir), Some(runs)) = (data_dir(&handle), state.runs.file_runs(&summary.id)) {
            runs::save_history(&dir, &summary, &runs);
        }
        Ok(summary)
    })
    .await?
}

#[command]
fn cancel_run(state: State<'_, AppState>, run_id: String) {
    state.runs.cancel(&run_id);
}

#[command]
fn get_response_body(
    state: State<'_, AppState>,
    run_id: String,
    file: usize,
    entry: usize,
    call: usize,
) -> Result<String> {
    Ok(state.runs.response_body(&run_id, file, entry, call)?)
}

/// JSONPath / XPath preview on a response kept in memory (the body never crosses IPC).
#[command]
fn query_body(
    state: State<'_, AppState>,
    run_id: String,
    file: usize,
    entry: usize,
    call: usize,
    kind: query::QueryKind,
    expr: String,
) -> Result<query::QueryOutcome> {
    let (body, content_type) = state.runs.response_bytes(&run_id, file, entry, call)?;
    Ok(query::query(kind, &body, content_type.as_deref(), &expr))
}

/// Same preview on text the frontend already has (history runs keep only the body preview).
#[command]
fn query_text(
    text: String,
    content_type: Option<String>,
    kind: query::QueryKind,
    expr: String,
) -> query::QueryOutcome {
    query::query(kind, text.as_bytes(), content_type.as_deref(), &expr)
}

/// Raw bytes of a response (base64) with its content type, for image / binary previews.
#[command]
fn get_response_body_bytes(
    state: State<'_, AppState>,
    run_id: String,
    file: usize,
    entry: usize,
    call: usize,
) -> Result<BodyBytes> {
    const LIMIT: usize = 8 * 1024 * 1024;
    let (body, content_type) = state.runs.response_bytes(&run_id, file, entry, call)?;
    use base64::Engine;
    Ok(BodyBytes {
        truncated: body.len() > LIMIT,
        base64: base64::engine::general_purpose::STANDARD.encode(&body[..body.len().min(LIMIT)]),
        content_type,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BodyBytes {
    base64: String,
    content_type: Option<String>,
    truncated: bool,
}

/// Adds asserts / captures to entry `entry` (1-based) of `text` without reformatting anything else.
#[command]
fn add_response_items(
    text: String,
    entry: usize,
    asserts: Vec<model::AssertM>,
    captures: Vec<model::CaptureM>,
) -> Result<edit::Edit> {
    Ok(edit::add_response_items(&text, entry, &asserts, &captures)?)
}

/// Sets the expected body of entry `entry` (1-based) of `text` to `body`, byte for byte.
#[command]
fn set_response_body(text: String, entry: usize, body: String) -> Result<edit::Edit> {
    Ok(edit::set_response_body(&text, entry, &body)?)
}

/// Converts a cURL / Postman / OpenCollection source into HURL files without writing anything.
#[command]
fn import_preview(
    kind: import::ImportKind,
    source: import::ImportSource,
    options: import::ImportOptions,
) -> Result<import::ImportResult> {
    Ok(import::import(kind, &source, &options)?)
}

/// Same conversion, written below `target_dir` (inside the workspace); refuses to overwrite.
#[command]
fn import_apply(
    state: State<'_, AppState>,
    kind: import::ImportKind,
    source: import::ImportSource,
    options: import::ImportOptions,
    target_dir: String,
) -> Result<Vec<String>> {
    let root = state.root()?;
    let result = import::import(kind, &source, &options)?;
    let paths = import::write(&result, &root, Path::new(&target_dir))?;
    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[command]
fn list_runs(app: AppHandle) -> Result<Vec<runs::RunSummary>> {
    Ok(runs::list_history(&data_dir(&app)?))
}

#[command]
fn load_run(app: AppHandle, run_id: String) -> Result<serde_json::Value> {
    Ok(runs::load_history_run(&data_dir(&app)?, &run_id)?)
}

#[command]
fn clear_runs(app: AppHandle) -> Result<()> {
    runs::delete_history(&data_dir(&app)?);
    Ok(())
}

/// Writes HURL's own report (`html`, `junit`, `tap`, `json`) and returns the main file path.
#[command]
fn export_report(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    format: String,
    dir: Option<String>,
    open: bool,
) -> Result<String> {
    let target = match dir {
        Some(d) => PathBuf::from(d),
        None => data_dir(&app)?
            .join("exports")
            .join(format!("{run_id}-{format}")),
    };
    let _ = std::fs::remove_dir_all(&target);
    let file = state.runs.export(&run_id, &format, &target)?;
    if open {
        app.opener()
            .open_path(file.to_string_lossy().to_string(), None::<&str>)?;
    }
    Ok(file.to_string_lossy().to_string())
}

// --------------------------------------------------------------- settings

#[command]
fn get_settings(app: AppHandle) -> serde_json::Value {
    data_dir(&app)
        .ok()
        .and_then(|d| std::fs::read_to_string(d.join("settings.json")).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(serde_json::Value::Null)
}

#[command]
fn set_settings(app: AppHandle, settings: serde_json::Value) -> Result<()> {
    let dir = data_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(Error::io("cannot create", &dir))?;
    let file = dir.join("settings.json");
    std::fs::write(&file, settings.to_string()).map_err(Error::io("cannot write", &file))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                if let Some(win) = _app.get_webview_window("main") {
                    let _ = apply_vibrancy(&win, NSVisualEffectMaterial::Sidebar, None, None);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hurl_version,
            parse_hurl,
            emit_hurl,
            format_hurl,
            open_workspace,
            close_workspace,
            list_dir,
            create_file,
            create_dir,
            rename_path,
            delete_path,
            duplicate_path,
            get_dir_vars,
            set_dir_vars,
            get_env_text,
            set_env_text,
            get_effective_vars,
            get_headers_text,
            set_headers_text,
            get_effective_headers,
            read_text_file,
            write_text_file,
            open_file_dialog,
            open_folder_dialog,
            pick_data_file,
            check_data_file,
            run_scope,
            cancel_run,
            get_response_body,
            get_response_body_bytes,
            query_body,
            query_text,
            add_response_items,
            set_response_body,
            import_preview,
            import_apply,
            list_runs,
            load_run,
            clear_runs,
            export_report,
            get_settings,
            set_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
