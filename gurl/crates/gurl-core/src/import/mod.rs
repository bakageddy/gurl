//! Importers: cURL commands, Postman collections (v2.1) and OpenCollection YAML, all producing the
//! same [`HurlDoc`] model the builder edits, laid out as folders and `.hurl` files plus `gurl.env`.

pub mod curl;
pub mod opencollection;
pub mod postman;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::emit;
use crate::model::{EntryM, HurlDoc};
use crate::workspace::{EnvVar, emit_env};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportKind {
    Curl,
    Postman,
    Opencollection,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("{0}")]
    Invalid(String),
    #[error("cannot read {}: {source}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{} already exists", .0.display())]
    Exists(PathBuf),
    #[error(transparent)]
    Workspace(#[from] crate::error::WorkspaceError),
}

/// What to import: pasted text or a file / directory.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSource {
    pub text: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportOptions {
    /// One `.hurl` per folder (requests concatenated) instead of one per request.
    pub one_file: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportedFile {
    /// Path relative to the import target, `/`-separated.
    pub rel_path: String,
    pub doc: HurlDoc,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportedEnv {
    pub rel_dir: String,
    /// `gurl.env` or `gurl.env.<environment>` for additional environments.
    pub file_name: String,
    pub vars: Vec<EnvVar>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub files: Vec<ImportedFile>,
    pub envs: Vec<ImportedEnv>,
    pub warnings: Vec<String>,
}

// ----------------------------------------------------------- intermediate

/// A request as importers see it, before layout.
#[derive(Debug, Clone, Default)]
pub struct Request {
    pub name: String,
    pub entry: EntryM,
}

/// A folder of requests; the root folder's name is the collection name.
#[derive(Debug, Clone, Default)]
pub struct Folder {
    pub name: String,
    pub requests: Vec<Request>,
    pub folders: Vec<Folder>,
    pub vars: Vec<EnvVar>,
}

impl Folder {
    pub fn new(name: &str) -> Self {
        Folder {
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn is_empty(&self) -> bool {
        self.requests.is_empty()
            && self.folders.iter().all(Folder::is_empty)
            && self.vars.is_empty()
    }
}

/// Turns a folder tree into files. The root folder's own requests land at the top level; each
/// sub-folder becomes a directory. With `one_file`, a folder's requests share one file named after
/// the folder (captures then chain between them, like a hand-written scenario).
pub fn layout(root: &Folder, options: &ImportOptions, out: &mut ImportResult) {
    fn walk(folder: &Folder, dir: &str, options: &ImportOptions, out: &mut ImportResult) {
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        if !folder.vars.is_empty() {
            out.envs.push(ImportedEnv {
                rel_dir: dir.to_string(),
                file_name: "gurl.env".to_string(),
                vars: folder.vars.clone(),
            });
        }
        if options.one_file && !folder.requests.is_empty() {
            let name = slug(if dir.is_empty() {
                &folder.name
            } else {
                folder.name.as_str()
            });
            let doc = HurlDoc {
                entries: folder.requests.iter().map(|r| r.entry.clone()).collect(),
                trailing_comments: vec![],
            };
            push_file(out, &format!("{prefix}{name}.hurl"), doc);
        } else {
            let mut used: Vec<String> = Vec::new();
            for r in &folder.requests {
                let name = unique(&slug(&r.name), &mut used);
                let doc = HurlDoc {
                    entries: vec![r.entry.clone()],
                    trailing_comments: vec![],
                };
                push_file(out, &format!("{prefix}{name}.hurl"), doc);
            }
        }
        let mut used_dirs: Vec<String> = Vec::new();
        for f in &folder.folders {
            if f.is_empty() {
                continue;
            }
            let name = unique(&slug(&f.name), &mut used_dirs);
            walk(f, &format!("{prefix}{name}"), options, out);
        }
    }
    walk(root, "", options, out);
}

fn push_file(out: &mut ImportResult, rel_path: &str, doc: HurlDoc) {
    let text = emit::emit(&doc);
    out.files.push(ImportedFile {
        rel_path: rel_path.to_string(),
        doc,
        text,
    });
}

/// A file-system friendly name: lowercase ASCII words joined by `-`.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "request".to_string()
    } else {
        out
    }
}

fn unique(base: &str, used: &mut Vec<String>) -> String {
    let mut name = base.to_string();
    let mut n = 2;
    while used.contains(&name) {
        name = format!("{base}-{n}");
        n += 1;
    }
    used.push(name.clone());
    name
}

/// Postman-style `{{var}}` is already HURL syntax; `:param` path variables become `{{param}}`.
pub fn path_params_to_templates(url: &str) -> String {
    let (base, rest) = match url.find(['?', '#']) {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, ""),
    };
    let mut out = String::new();
    let mut after_slash = false;
    for ch in base.chars() {
        if ch == ':' && after_slash {
            out.push_str("{{");
            out.push('\u{0}');
        } else {
            out.push(ch);
        }
        after_slash = ch == '/';
    }
    // Close every opened template at the end of its path segment.
    let mut result = String::new();
    let mut open = false;
    for ch in out.chars() {
        if ch == '\u{0}' {
            open = true;
            continue;
        }
        if open && (ch == '/' || ch == '.') {
            result.push_str("}}");
            open = false;
        }
        result.push(ch);
    }
    if open {
        result.push_str("}}");
    }
    result + rest
}

// ------------------------------------------------------------------ entry

pub fn import(
    kind: ImportKind,
    source: &ImportSource,
    options: &ImportOptions,
) -> Result<ImportResult, ImportError> {
    match kind {
        ImportKind::Curl => {
            let text = source_text(source)?;
            curl::import(&text, options)
        }
        ImportKind::Postman => {
            let text = source_text(source)?;
            postman::import(&text, options)
        }
        ImportKind::Opencollection => match &source.path {
            Some(p) if Path::new(p).is_dir() => opencollection::import_dir(Path::new(p), options),
            _ => {
                let text = source_text(source)?;
                opencollection::import_text(&text, options)
            }
        },
    }
}

fn source_text(source: &ImportSource) -> Result<String, ImportError> {
    if let Some(t) = &source.text {
        return Ok(t.clone());
    }
    let path = source
        .path
        .as_deref()
        .ok_or_else(|| ImportError::Invalid("nothing to import".to_string()))?;
    std::fs::read_to_string(path).map_err(|e| ImportError::Read {
        path: PathBuf::from(path),
        source: e,
    })
}

/// Writes the result below `target` (inside `root`), refusing to overwrite. Returns the paths.
pub fn write(
    result: &ImportResult,
    root: &Path,
    target: &Path,
) -> Result<Vec<PathBuf>, ImportError> {
    let target = crate::workspace::contain(root, target)?;
    let mut planned: Vec<(PathBuf, String)> = Vec::new();
    for f in &result.files {
        planned.push((target.join(&f.rel_path), f.text.clone()));
    }
    for e in &result.envs {
        let dir = if e.rel_dir.is_empty() {
            target.clone()
        } else {
            target.join(&e.rel_dir)
        };
        planned.push((dir.join(&e.file_name), emit_env(&e.vars)));
    }
    for (path, _) in &planned {
        if path.exists() {
            return Err(ImportError::Exists(path.clone()));
        }
    }
    let mut written = Vec::new();
    for (path, text) in planned {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| ImportError::Read {
                path: dir.to_path_buf(),
                source: e,
            })?;
        }
        std::fs::write(&path, text).map_err(|e| ImportError::Read {
            path: path.clone(),
            source: e,
        })?;
        written.push(path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_path_params() {
        assert_eq!(slug("Get User (by id)!"), "get-user-by-id");
        assert_eq!(slug("  "), "request");
        assert_eq!(
            path_params_to_templates("https://api.example.com/users/:id/posts/:postId.json?x=1"),
            "https://api.example.com/users/{{id}}/posts/{{postId}}.json?x=1"
        );
        assert_eq!(
            path_params_to_templates("http://h:8080/a"),
            "http://h:8080/a"
        );
    }

    #[test]
    fn layout_one_file_per_request_or_per_folder() {
        let mut root = Folder::new("Shop");
        root.requests.push(Request {
            name: "List items".into(),
            ..Default::default()
        });
        let mut sub = Folder::new("Users");
        sub.requests.push(Request {
            name: "Login".into(),
            ..Default::default()
        });
        sub.requests.push(Request {
            name: "Login".into(),
            ..Default::default()
        });
        sub.vars.push(EnvVar {
            name: "host".into(),
            value: "x".into(),
            secret: false,
        });
        root.folders.push(sub);

        let mut out = ImportResult::default();
        layout(&root, &ImportOptions::default(), &mut out);
        let paths: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(
            paths,
            ["list-items.hurl", "users/login.hurl", "users/login-2.hurl"]
        );
        assert_eq!(out.envs[0].rel_dir, "users");

        let mut out = ImportResult::default();
        layout(&root, &ImportOptions { one_file: true }, &mut out);
        let paths: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(paths, ["shop.hurl", "users/users.hurl"]);
        assert_eq!(out.files[1].doc.entries.len(), 2);
    }
}
