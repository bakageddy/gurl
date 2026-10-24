//! Captures HURL's version from the vendored crate manifest and the compile target, for
//! `gurl --version`.

use std::path::Path;

fn main() {
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/hurl/packages/hurl/Cargo.toml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest)
        .expect("vendor/hurl submodule is missing: run `git submodule update --init`");
    let version = text
        .lines()
        .find_map(|l| l.strip_prefix("version = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .expect("no version in hurl's Cargo.toml");
    println!("cargo:rustc-env=GURL_HURL_VERSION={version}");
    println!(
        "cargo:rustc-env=GURL_TARGET={}",
        std::env::var("TARGET").unwrap_or_default()
    );
}
