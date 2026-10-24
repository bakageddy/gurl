// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::OsString;
use std::process::exit;

/// `gurl` opens the app; `gurl cli …`, `gurl --version` and `gurl --help` run headless, before any
/// Tauri initialisation (no event loop, no window, no console allocation).
fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    match args.get(1).and_then(|a| a.to_str()) {
        Some("cli") | Some("hurl") => {
            attach_console();
            exit(gurl_core::cli::hurl::main(&args[2..]));
        }
        Some("--version") | Some("-V") => {
            attach_console();
            print!("{}", gurl_core::cli::version::long());
            exit(0);
        }
        Some("--help") | Some("-h") => {
            attach_console();
            print!("{}", gurl_core::cli::help());
            exit(0);
        }
        _ => gurl_lib::run(),
    }
}

/// On Windows a GUI-subsystem process has no console; attach to the parent's so that stdout and
/// stderr reach the terminal that started `gurl cli`. Redirections and pipes already work without
/// this. Nothing to do elsewhere.
fn attach_console() {
    #[cfg(windows)]
    // SAFETY: plain Win32 call with a constant argument; failure (no parent console) is harmless.
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
