//! gURL engine: the HURL document model, workspaces, variable inheritance, runs and reports.
//! Free of any GUI dependency so that both the desktop app and the CLI build on it.

pub mod cli;
pub mod edit;
pub mod emit;
pub mod error;
pub mod hurl;
pub mod model;
pub mod parse;
pub mod query;
pub mod runs;
pub mod workspace;

// hurl's RFC 9535 JSONPath, private in the hurl crate but self-contained: compile it in from the
// vendored sources so the app's previews use exactly hurl's evaluator.
#[allow(dead_code, clippy::all)]
#[path = "../../../vendor/hurl/packages/hurl/src/jsonpath/mod.rs"]
mod jsonpath;
