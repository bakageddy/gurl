//! Workspace model: a root directory that can nest up to [`MAX_DEPTH`] levels. Every
//! directory may hold many `.hurl` scenarios and one `zurl.env` file with variable
//! definitions/overrides that apply to its scenarios and to every sub-directory.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

pub const MAX_DEPTH: usize = 255;
pub const ENV_FILE: &str = "zurl.env";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub name: String,
    pub path: String,
    /// `dir`, `hurl` or `env`.
    pub kind: &'static str,
    /// For dirs: whether the dir contains a `zurl.env`.
    pub has_env: bool,
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
    /// Directory whose `zurl.env` defines the winning value.
    pub source_dir: String,
    /// Directories further up that define the same name and are overridden.
    pub overrides: Vec<String>,
}

/// Number of directory levels `path` sits below `root` (root itself is depth 0).
pub fn depth_of(root: &Path, path: &Path) -> Result<usize, String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside the workspace", path.display()))?;
    Ok(rel
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count())
}

/// Resolves `path` (which may not exist yet) and makes sure it lives under `root` without `..`
/// tricks. Returns the lexically-normalised absolute path.
pub fn contain(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                if !out.pop() {
                    return Err("path escapes the workspace".into());
                }
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    if !out.starts_with(root) {
        return Err(format!("{} is outside the workspace", out.display()));
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
            _ => 2,
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

/// Lists one directory level (dirs, `.hurl` files and `zurl.env`); dotfiles are hidden.
pub fn list_dir(root: &Path, dir: &Path) -> Result<Vec<Node>, String> {
    let dir = contain(root, dir)?;
    if depth_of(root, &dir)? > MAX_DEPTH {
        return Err(format!("maximum nesting depth of {MAX_DEPTH} exceeded"));
    }
    let mut nodes = Vec::new();
    let rd = std::fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
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
        } else if is_hurl(&name) {
            "hurl"
        } else {
            continue;
        };
        let has_env = kind == "dir" && path.join(ENV_FILE).is_file();
        nodes.push(Node {
            name,
            path: path.to_string_lossy().to_string(),
            kind,
            has_env,
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

pub fn write_env(root: &Path, dir: &Path, vars: &[EnvVar]) -> Result<(), String> {
    let dir = contain(root, dir)?;
    let file = dir.join(ENV_FILE);
    if vars.iter().all(|v| v.name.trim().is_empty()) {
        if file.exists() {
            std::fs::remove_file(&file).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    std::fs::write(&file, emit_env(vars)).map_err(|e| format!("cannot write {}: {e}", file.display()))
}

/// Directories from `root` down to `dir`, inclusive.
fn chain(root: &Path, dir: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = contain(root, dir)?;
    let depth = depth_of(root, &dir)?;
    if depth > MAX_DEPTH {
        return Err(format!("maximum nesting depth of {MAX_DEPTH} exceeded"));
    }
    let mut dirs: Vec<PathBuf> = dir
        .ancestors()
        .take(depth + 1)
        .map(Path::to_path_buf)
        .collect();
    dirs.reverse();
    Ok(dirs)
}

/// Variables as seen by scenarios in `dir`: root first, each deeper `zurl.env` overriding.
pub fn effective_vars(root: &Path, dir: &Path) -> Result<Vec<EffectiveVar>, String> {
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

// ----------------------------------------------------------------- file ops

fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || name.contains('\0')
    {
        return Err(format!("invalid name: {name:?}"));
    }
    Ok(())
}

pub fn create_file(root: &Path, dir: &Path, name: &str, content: &str) -> Result<PathBuf, String> {
    valid_name(name)?;
    let dir = contain(root, dir)?;
    let path = dir.join(name);
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn create_dir(root: &Path, parent: &Path, name: &str) -> Result<PathBuf, String> {
    valid_name(name)?;
    let parent = contain(root, parent)?;
    let path = parent.join(name);
    if depth_of(root, &path)? > MAX_DEPTH {
        return Err(format!("maximum nesting depth of {MAX_DEPTH} exceeded"));
    }
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    std::fs::create_dir(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn rename(root: &Path, from: &Path, new_name: &str) -> Result<PathBuf, String> {
    valid_name(new_name)?;
    let from = contain(root, from)?;
    if from == root {
        return Err("cannot rename the workspace root".into());
    }
    let to = from.with_file_name(new_name);
    if to.exists() {
        return Err(format!("{} already exists", to.display()));
    }
    std::fs::rename(&from, &to).map_err(|e| e.to_string())?;
    Ok(to)
}

pub fn delete(root: &Path, path: &Path) -> Result<(), String> {
    let path = contain(root, path)?;
    if path == root {
        return Err("cannot delete the workspace root".into());
    }
    let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if meta.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(&path).map_err(|e| e.to_string())
    }
}

pub fn duplicate(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let path = contain(root, path)?;
    if !path.is_file() {
        return Err("only files can be duplicated".into());
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
            std::fs::copy(&path, &to).map_err(|e| e.to_string())?;
            return Ok(to);
        }
    }
    unreachable!()
}

/// All `.hurl` files below `dir` (recursively when `recursive`), sorted, depth-limited.
pub fn collect_hurl_files(root: &Path, dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), String> {
        if depth_of(root, dir)? > MAX_DEPTH {
            return Ok(());
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
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
pub fn data_file_relative(hurl_file: &Path, picked: &Path) -> Result<String, String> {
    let dir = hurl_file.parent().ok_or("scenario has no directory")?;
    let dir = normalize(dir);
    let picked = normalize(picked);
    let rel = picked.strip_prefix(&dir).map_err(|_| {
        format!(
            "data files must be inside {} (the scenario's directory) or a sub-directory",
            dir.display()
        )
    })?;
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/"))
}

pub fn check_data_file(hurl_file: &Path, reference: &str) -> DataFileStatus {
    let Some(dir) = hurl_file.parent() else {
        return DataFileStatus { exists: false, allowed: false };
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
        let p = std::env::temp_dir().join(format!("zurl-ws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn env_roundtrip_with_secret() {
        let vars = vec![
            EnvVar { name: "host".into(), value: "a.b".into(), secret: false },
            EnvVar { name: "token".into(), value: "x=y".into(), secret: true },
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
