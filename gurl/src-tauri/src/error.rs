//! Error types. Each layer owns the failures it can name; [`Error`] is what Tauri commands
//! return and serialises to the message string the frontend already displays.

use serde::{Serialize, Serializer};
use std::error::Error as StdError;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::workspace::MAX_DEPTH;

/// Joins an error and its `source()` chain into one line: `outer: inner: root cause`.
pub fn chain_message(err: &dyn StdError) -> String {
    let mut out = err.to_string();
    let mut cur = err.source();
    while let Some(e) = cur {
        out.push_str(": ");
        out.push_str(&e.to_string());
        cur = e.source();
    }
    out
}

/// Failures of workspace file and path handling.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("{} is outside the workspace", .0.display())]
    Outside(PathBuf),
    #[error("path escapes the workspace")]
    Escapes,
    #[error("maximum nesting depth of {MAX_DEPTH} exceeded")]
    TooDeep,
    #[error("invalid name: {0:?}")]
    InvalidName(String),
    #[error("{} already exists", .0.display())]
    AlreadyExists(PathBuf),
    #[error("cannot {0} the workspace root")]
    RootProtected(&'static str),
    #[error("only files can be duplicated")]
    NotAFile,
    #[error("scenario has no directory")]
    NoParentDir,
    #[error("data files must be inside {} (the scenario's directory) or a sub-directory", .0.display())]
    DataFileOutside(PathBuf),
    #[error("{action} {}", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl WorkspaceError {
    /// For `map_err`: `fs::read_dir(&d).map_err(WorkspaceError::io("cannot read", &d))`.
    pub fn io(action: &'static str, path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { action, path, source }
    }
}

/// Failures of running scenarios, keeping run state and exporting reports.
#[derive(Debug, Error)]
pub enum RunError {
    #[error("missing path")]
    MissingPath,
    #[error("unknown run kind: {0}")]
    UnknownKind(String),
    #[error("unknown report format: {0}")]
    UnknownFormat(String),
    #[error("run is no longer in memory; re-run it")]
    NotInMemory,
    #[error("invalid run id")]
    InvalidRunId,
    #[error("invalid value for option `{name}`: {reason}")]
    InvalidOption { name: &'static str, reason: String },
    /// HURL's report writers return several unrelated error types; only their text is useful.
    #[error("cannot write report: {0}")]
    Report(String),
    #[error("{action} {}", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid run data")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

impl RunError {
    pub fn io(action: &'static str, path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { action, path, source }
    }

    pub fn report(err: impl std::fmt::Display) -> Self {
        Self::Report(err.to_string())
    }
}

/// The error type of every Tauri command.
#[derive(Debug, Error)]
pub enum Error {
    #[error("no workspace is open")]
    NoWorkspace,
    #[error("{} is not a directory", .0.display())]
    NotADirectory(PathBuf),
    #[error("only .hurl files can be accessed outside the workspace")]
    OutsideAccess,
    #[error("{action} {}", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Run(#[from] RunError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("selected file is not a local path")]
    NotLocalPath(#[source] Box<dyn StdError + Send + Sync>),
    #[error(transparent)]
    Opener(#[from] tauri_plugin_opener::Error),
}

impl Error {
    pub fn io(action: &'static str, path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { action, path, source }
    }
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&chain_message(self))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_the_whole_source_chain() {
        let err: Error = WorkspaceError::io(
            "cannot read",
            Path::new("/w/a"),
        )(io::Error::new(io::ErrorKind::NotFound, "gone"))
        .into();
        assert_eq!(serde_json::to_string(&err).unwrap(), "\"cannot read /w/a: gone\"");
    }

    #[test]
    fn workspace_errors_keep_their_messages() {
        let msg = |e: WorkspaceError| Error::from(e).to_string();
        assert_eq!(msg(WorkspaceError::TooDeep), "maximum nesting depth of 255 exceeded");
        assert_eq!(msg(WorkspaceError::RootProtected("delete")), "cannot delete the workspace root");
        assert_eq!(msg(WorkspaceError::InvalidName("a/b".into())), "invalid name: \"a/b\"");
    }
}
