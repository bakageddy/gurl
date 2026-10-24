//! The error type of every Tauri command. It serialises to one message string (cause chain
//! included), which is what the frontend displays.

use gurl_core::error::{chain_message, RunError, WorkspaceError};
use serde::{Serialize, Serializer};
use std::error::Error as StdError;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
        move |source| Self::Io {
            action,
            path,
            source,
        }
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
        let err: Error = WorkspaceError::io("cannot read", Path::new("/w/a"))(io::Error::new(
            io::ErrorKind::NotFound,
            "gone",
        ))
        .into();
        assert_eq!(
            serde_json::to_string(&err).unwrap(),
            "\"cannot read /w/a: gone\""
        );
    }

    #[test]
    fn workspace_errors_keep_their_messages() {
        let msg = |e: WorkspaceError| Error::from(e).to_string();
        assert_eq!(
            msg(WorkspaceError::TooDeep),
            "maximum nesting depth of 255 exceeded"
        );
        assert_eq!(
            msg(WorkspaceError::RootProtected("delete")),
            "cannot delete the workspace root"
        );
        assert_eq!(
            msg(WorkspaceError::InvalidName("a/b".into())),
            "invalid name: \"a/b\""
        );
    }
}
