//! Workspace model: a root directory that can nest up to [`MAX_DEPTH`] levels. Every
//! directory may hold many `.hurl` scenarios and one `gurl.env` file with variable
//! definitions/overrides that apply to its scenarios and to every sub-directory.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

use crate::error::WorkspaceError;

type Result<T> = std::result::Result<T, WorkspaceError>;

pub const MAX_DEPTH: usize = 255;
pub const ENV_FILE: &str = "gurl.env";
/// Headers added to every request of the folder's scenarios and sub-folders (`Name: value` lines,
/// values may use `{{var}}`). A deeper folder overrides a header of the same name.
pub const HEADERS_FILE: &str = "gurl.headers";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub name: String,
    pub path: String,
    /// `dir`, `hurl`, `env` or `headers`.
    pub kind: &'static str,
    /// For dirs: whether the dir contains a `gurl.env`.
    pub has_env: bool,
    /// For dirs: whether the dir contains a `gurl.headers`.
    pub has_headers: bool,
}

/// One line of a `gurl.headers` file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HeaderLine {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveHeader {
    pub name: String,
    pub value: String,
    /// Directory whose `gurl.headers` defines the winning value.
    pub source_dir: String,
    /// Directories further up that define the same header and are overridden.
    pub overrides: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub secret: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveVar {
    pub name: String,
    pub value: String,
    pub secret: bool,
    /// Directory whose `gurl.env` defines the winning value.
    pub source_dir: String,
    /// Directories further up that define the same name and are overridden.
    pub overrides: Vec<String>,
}

/// Number of directory levels `path` sits below `root` (root itself is depth 0).
pub fn depth_of(root: &Path, path: &Path) -> Result<usize> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| WorkspaceError::Outside(path.to_path_buf()))?;
    Ok(rel
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count())
}

/// Resolves `path` (which may not exist yet) and makes sure it lives under `root` without `..`
/// tricks. Returns the lexically-normalised absolute path.
pub fn contain(root: &Path, path: &Path) -> Result<PathBuf> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                if !out.pop() {
                    return Err(WorkspaceError::Escapes);
                }
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    if !out.starts_with(root) {
        return Err(WorkspaceError::Outside(out));
    }
    Ok(out)
}

fn is_hurl(name: &str) -> bool {
    name.ends_with(".hurl") || name.ends_with(".hur")
}

fn sort_nodes(nodes: &mut [Node]) {
    nodes.sort_by(|a, b| {
        let rank = |n: &Node| match n.kind {
            "dir" => 0,
            "env" => 1,
            "headers" => 2,
            _ => 3,
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

/// Lists one directory level (dirs, `.hurl` files and `gurl.env`); dotfiles are hidden.
pub fn list_dir(root: &Path, dir: &Path) -> Result<Vec<Node>> {
    let dir = contain(root, dir)?;
    if depth_of(root, &dir)? > MAX_DEPTH {
        return Err(WorkspaceError::TooDeep);
    }
    let mut nodes = Vec::new();
    let rd = std::fs::read_dir(&dir).map_err(WorkspaceError::io("cannot read", &dir))?;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        // `metadata` follows symlinks; a symlinked dir is listed but never recursed into
        // by this function, so loops can't hang the UI.
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let kind = if meta.is_dir() {
            "dir"
        } else if name == ENV_FILE {
            "env"
        } else if name == HEADERS_FILE {
            "headers"
        } else if is_hurl(&name) {
            "hurl"
        } else {
            continue;
        };
        let has_env = kind == "dir" && path.join(ENV_FILE).is_file();
        let has_headers = kind == "dir" && path.join(HEADERS_FILE).is_file();
        nodes.push(Node {
            name,
            path: path.to_string_lossy().to_string(),
            kind,
            has_env,
            has_headers,
        });
    }
    sort_nodes(&mut nodes);
    Ok(nodes)
}

// ---------------------------------------------------------------- env files

pub fn parse_env(text: &str) -> Vec<EnvVar> {
    let mut vars = Vec::new();
    let mut secret = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(c) = t.strip_prefix('#') {
            secret = c.trim() == "@secret";
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            vars.push(EnvVar {
                name: k.trim().to_string(),
                value: v.trim().to_string(),
                secret,
            });
        }
        secret = false;
    }
    vars
}

pub fn emit_env(vars: &[EnvVar]) -> String {
    let mut out = String::new();
    for v in vars.iter().filter(|v| !v.name.trim().is_empty()) {
        if v.secret {
            out.push_str("# @secret\n");
        }
        out.push_str(&format!("{}={}\n", v.name.trim(), v.value));
    }
    out
}

pub fn read_env(dir: &Path) -> Vec<EnvVar> {
    std::fs::read_to_string(dir.join(ENV_FILE))
        .map(|t| parse_env(&t))
        .unwrap_or_default()
}

pub fn write_env(root: &Path, dir: &Path, vars: &[EnvVar]) -> Result<()> {
    let dir = contain(root, dir)?;
    let file = dir.join(ENV_FILE);
    if vars.iter().all(|v| v.name.trim().is_empty()) {
        if file.exists() {
            std::fs::remove_file(&file).map_err(WorkspaceError::io("cannot remove", &file))?;
        }
        return Ok(());
    }
    std::fs::write(&file, emit_env(vars)).map_err(WorkspaceError::io("cannot write", &file))
}

/// Raw text of a directory's `gurl.env` (empty when there is none).
pub fn read_env_text(root: &Path, dir: &Path) -> Result<String> {
    let dir = contain(root, dir)?;
    Ok(std::fs::read_to_string(dir.join(ENV_FILE)).unwrap_or_default())
}

/// Writes raw `gurl.env` text; blank text removes the file.
pub fn write_env_text(root: &Path, dir: &Path, text: &str) -> Result<()> {
    let dir = contain(root, dir)?;
    let file = dir.join(ENV_FILE);
    if text.trim().is_empty() {
        if file.exists() {
            std::fs::remove_file(&file).map_err(WorkspaceError::io("cannot remove", &file))?;
        }
        return Ok(());
    }
    std::fs::write(&file, text).map_err(WorkspaceError::io("cannot write", &file))
}

/// Directories from `root` down to `dir`, inclusive.
fn chain(root: &Path, dir: &Path) -> Result<Vec<PathBuf>> {
    let dir = contain(root, dir)?;
    let depth = depth_of(root, &dir)?;
    if depth > MAX_DEPTH {
        return Err(WorkspaceError::TooDeep);
    }
    let mut dirs: Vec<PathBuf> = dir
        .ancestors()
        .take(depth + 1)
        .map(Path::to_path_buf)
        .collect();
    dirs.reverse();
    Ok(dirs)
}

/// Variables as seen by scenarios in `dir`: root first, each deeper `gurl.env` overriding.
pub fn effective_vars(root: &Path, dir: &Path) -> Result<Vec<EffectiveVar>> {
    let mut out: Vec<EffectiveVar> = Vec::new();
    for d in chain(root, dir)? {
        let d_str = d.to_string_lossy().to_string();
        for v in read_env(&d) {
            if let Some(existing) = out.iter_mut().find(|e| e.name == v.name) {
                existing.overrides.push(existing.source_dir.clone());
                existing.value = v.value;
                existing.secret = v.secret;
                existing.source_dir = d_str.clone();
            } else {
                out.push(EffectiveVar {
                    name: v.name,
                    value: v.value,
                    secret: v.secret,
                    source_dir: d_str.clone(),
                    overrides: vec![],
                });
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------------ headers files

/// Parses `gurl.headers`: `Name: value` per line, `#` comments and blank lines ignored. A line
/// without `:` is skipped (the editor shows the raw text, so nothing is silently rewritten).
pub fn parse_headers(text: &str) -> Vec<HeaderLine> {
    text.lines()
        .map(str::trim)
        .filter(|t| !t.is_empty() && !t.starts_with('#'))
        .filter_map(|t| t.split_once(':'))
        .filter(|(k, _)| !k.trim().is_empty())
        .map(|(k, v)| HeaderLine {
            name: k.trim().to_string(),
            value: v.trim().to_string(),
        })
        .collect()
}

pub fn read_headers(dir: &Path) -> Vec<HeaderLine> {
    std::fs::read_to_string(dir.join(HEADERS_FILE))
        .map(|t| parse_headers(&t))
        .unwrap_or_default()
}

/// Raw text of a directory's `gurl.headers` (empty when there is none).
pub fn read_headers_text(root: &Path, dir: &Path) -> Result<String> {
    let dir = contain(root, dir)?;
    Ok(std::fs::read_to_string(dir.join(HEADERS_FILE)).unwrap_or_default())
}

/// Writes raw `gurl.headers` text; blank text removes the file.
pub fn write_headers_text(root: &Path, dir: &Path, text: &str) -> Result<()> {
    let dir = contain(root, dir)?;
    let file = dir.join(HEADERS_FILE);
    if text.trim().is_empty() {
        if file.exists() {
            std::fs::remove_file(&file).map_err(WorkspaceError::io("cannot remove", &file))?;
        }
        return Ok(());
    }
    std::fs::write(&file, text).map_err(WorkspaceError::io("cannot write", &file))
}

/// Headers added to requests of scenarios in `dir`: root first, each deeper `gurl.headers`
/// overriding by (case-insensitive) name.
pub fn effective_headers(root: &Path, dir: &Path) -> Result<Vec<EffectiveHeader>> {
    let mut out: Vec<EffectiveHeader> = Vec::new();
    for d in chain(root, dir)? {
        let d_str = d.to_string_lossy().to_string();
        for h in read_headers(&d) {
            if let Some(existing) = out.iter_mut().find(|e| e.name.eq_ignore_ascii_case(&h.name)) {
                existing.overrides.push(existing.source_dir.clone());
                existing.name = h.name;
                existing.value = h.value;
                existing.source_dir = d_str.clone();
            } else {
                out.push(EffectiveHeader {
                    name: h.name,
                    value: h.value,
                    source_dir: d_str.clone(),
                    overrides: vec![],
                });
            }
        }
    }
    Ok(out)
}

// ----------------------------------------------------------------- file ops

fn valid_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || name.contains('\0')
    {
        return Err(WorkspaceError::InvalidName(name.to_string()));
    }
    Ok(())
}

pub fn create_file(root: &Path, dir: &Path, name: &str, content: &str) -> Result<PathBuf> {
    valid_name(name)?;
    let dir = contain(root, dir)?;
    let path = dir.join(name);
    if path.exists() {
        return Err(WorkspaceError::AlreadyExists(path));
    }
    std::fs::write(&path, content).map_err(WorkspaceError::io("cannot write", &path))?;
    Ok(path)
}

pub fn create_dir(root: &Path, parent: &Path, name: &str) -> Result<PathBuf> {
    valid_name(name)?;
    let parent = contain(root, parent)?;
    let path = parent.join(name);
    if depth_of(root, &path)? > MAX_DEPTH {
        return Err(WorkspaceError::TooDeep);
    }
    if path.exists() {
        return Err(WorkspaceError::AlreadyExists(path));
    }
    std::fs::create_dir(&path).map_err(WorkspaceError::io("cannot create", &path))?;
    Ok(path)
}

pub fn rename(root: &Path, from: &Path, new_name: &str) -> Result<PathBuf> {
    valid_name(new_name)?;
    let from = contain(root, from)?;
    if from == root {
        return Err(WorkspaceError::RootProtected("rename"));
    }
    let to = from.with_file_name(new_name);
    if to.exists() {
        return Err(WorkspaceError::AlreadyExists(to));
    }
    std::fs::rename(&from, &to).map_err(WorkspaceError::io("cannot rename", &from))?;
    Ok(to)
}

pub fn delete(root: &Path, path: &Path) -> Result<()> {
    let path = contain(root, path)?;
    if path == root {
        return Err(WorkspaceError::RootProtected("delete"));
    }
    let meta =
        std::fs::symlink_metadata(&path).map_err(WorkspaceError::io("cannot read", &path))?;
    let removed = if meta.is_dir() {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_file(&path)
    };
    removed.map_err(WorkspaceError::io("cannot delete", &path))
}

pub fn duplicate(root: &Path, path: &Path) -> Result<PathBuf> {
    let path = contain(root, path)?;
    if !path.is_file() {
        return Err(WorkspaceError::NotAFile);
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("copy");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    for n in 1.. {
        let name = if ext.is_empty() {
            format!("{stem} copy {n}")
        } else {
            format!("{stem} copy {n}.{ext}")
        };
        let to = path.with_file_name(name);
        if !to.exists() {
            std::fs::copy(&path, &to).map_err(WorkspaceError::io("cannot copy", &path))?;
            return Ok(to);
        }
    }
    unreachable!()
}

/// All `.hurl` files below `dir` (recursively when `recursive`), sorted, depth-limited.
pub fn collect_hurl_files(root: &Path, dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    fn walk(root: &Path, dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<()> {
        if depth_of(root, dir)? > MAX_DEPTH {
            return Ok(());
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(WorkspaceError::io("cannot read", dir))?
            .flatten()
            .collect();
        entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let path = e.path();
            // Don't follow symlinked dirs: avoids cycles.
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if recursive {
                    walk(root, &path, recursive, out)?;
                }
            } else if is_hurl(&name) {
                out.push(path);
            }
        }
        Ok(())
    }
    let dir = contain(root, dir)?;
    let mut out = Vec::new();
    walk(root, &dir, recursive, &mut out)?;
    Ok(out)
}

/// Result of validating a `file,...;` reference (multipart parts and bodies).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFileStatus {
    pub exists: bool,
    /// Inside the scenario's directory (HURL's default file root) and thus runnable.
    pub allowed: bool,
}

/// Lexically normalises `path` (`.` and `..`) without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Data files referenced from a `.hurl` file must live in that file's directory or below
/// (HURL's file root). Returns the `/`-separated relative path for `picked`.
pub fn data_file_relative(hurl_file: &Path, picked: &Path) -> Result<String> {
    let dir = hurl_file.parent().ok_or(WorkspaceError::NoParentDir)?;
    let dir = normalize(dir);
    let picked = normalize(picked);
    let rel = picked
        .strip_prefix(&dir)
        .map_err(|_| WorkspaceError::DataFileOutside(dir.clone()))?;
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/"))
}

pub fn check_data_file(hurl_file: &Path, reference: &str) -> DataFileStatus {
    let Some(dir) = hurl_file.parent() else {
        return DataFileStatus {
            exists: false,
            allowed: false,
        };
    };
    let dir = normalize(dir);
    let full = normalize(&dir.join(reference));
    DataFileStatus {
        exists: full.is_file(),
        allowed: full.starts_with(&dir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("gurl-ws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn env_roundtrip_with_secret() {
        let vars = vec![
            EnvVar {
                name: "host".into(),
                value: "a.b".into(),
                secret: false,
            },
            EnvVar {
                name: "token".into(),
                value: "x=y".into(),
                secret: true,
            },
        ];
        assert_eq!(parse_env(&emit_env(&vars)), vars);
    }

    #[test]
    fn child_overrides_parent() {
        let root = tmp("vars");
        let child = root.join("a").join("b");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(root.join(ENV_FILE), "host=root\nroot_only=1\n").unwrap();
        std::fs::write(root.join("a").join(ENV_FILE), "host=a\n").unwrap();
        std::fs::write(child.join(ENV_FILE), "host=b\n").unwrap();
        let vars = effective_vars(&root, &child).unwrap();
        let host = vars.iter().find(|v| v.name == "host").unwrap();
        assert_eq!(host.value, "b");
        assert_eq!(host.overrides.len(), 2);
        assert!(vars.iter().any(|v| v.name == "root_only"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn headers_parse_and_child_overrides_parent_case_insensitively() {
        assert_eq!(
            parse_headers("# auth\nAuthorization: Bearer {{token}}\n\nbad line\nX-Empty:\n"),
            vec![
                HeaderLine {
                    name: "Authorization".into(),
                    value: "Bearer {{token}}".into()
                },
                HeaderLine {
                    name: "X-Empty".into(),
                    value: String::new()
                }
            ]
        );
        let root = tmp("headers");
        let child = root.join("a");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(
            root.join(HEADERS_FILE),
            "Authorization: Bearer root\nX-Trace: 1\n",
        )
        .unwrap();
        std::fs::write(child.join(HEADERS_FILE), "authorization: Bearer a\n").unwrap();
        let hs = effective_headers(&root, &child).unwrap();
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].name, "authorization");
        assert_eq!(hs[0].value, "Bearer a");
        assert_eq!(hs[0].overrides.len(), 1);
        assert_eq!(hs[1].name, "X-Trace");
        let nodes = list_dir(&root, &root).unwrap();
        assert!(nodes.iter().any(|n| n.kind == "headers"));
        assert!(nodes.iter().any(|n| n.kind == "dir" && n.has_headers));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn depth_limit_is_255() {
        let root = tmp("depth");
        let mut cur = root.clone();
        for i in 0..MAX_DEPTH {
            cur = create_dir(&root, &cur, &format!("d{i}")).unwrap();
        }
        assert_eq!(depth_of(&root, &cur).unwrap(), 255);
        let err = create_dir(&root, &cur, "too-deep");
        assert!(err.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn data_files_must_stay_below_scenario_dir() {
        let hurl = Path::new("/w/api/a.hurl");
        assert_eq!(
            data_file_relative(hurl, Path::new("/w/api/data/x.png")).unwrap(),
            "data/x.png"
        );
        assert!(data_file_relative(hurl, Path::new("/w/other/x.png")).is_err());
        assert!(!check_data_file(hurl, "../secret.txt").allowed);
        assert!(check_data_file(hurl, "data/x.png").allowed);
    }

    #[test]
    fn rejects_path_escape() {
        let root = tmp("escape");
        assert!(contain(&root, &root.join("..").join("x")).is_err());
        assert!(create_file(&root, &root, "../evil.hurl", "").is_err());
        assert!(delete(&root, &root).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
