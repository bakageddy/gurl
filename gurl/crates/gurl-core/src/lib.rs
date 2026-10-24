//! gURL engine: the HURL document model, workspaces, variable inheritance, runs and reports.
//! Free of any GUI dependency so that both the desktop app and the CLI build on it.

pub mod cli;
pub mod emit;
pub mod error;
pub mod hurl;
pub mod model;
pub mod parse;
pub mod runs;
pub mod workspace;
