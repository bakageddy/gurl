//! No-op stand-in for the `winres` crate.
//!
//! HURL's `build.rs` embeds its own icon and VERSIONINFO into whatever binary links it. gURL's
//! binary already gets those from Tauri, and the two collide at link time on Windows
//! (`CVT1100: duplicate resource. type:VERSION, name:1` / `LNK1123`). Patching `winres` with this
//! stub (`[patch.crates-io]` in `src-tauri/Cargo.toml`) removes HURL's copy without editing the
//! vendored HURL sources. Only HURL depends on `winres`; Tauri uses `tauri-winres`.

use std::io;

pub struct WindowsResource;

impl WindowsResource {
    pub fn new() -> Self {
        WindowsResource
    }

    pub fn set_icon(&mut self, _path: &str) -> &mut Self {
        self
    }

    pub fn compile(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Default for WindowsResource {
    fn default() -> Self {
        Self::new()
    }
}
