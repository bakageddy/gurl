//! Everything `gurl` does from a terminal: `gurl cli` (HURL's CLI), `gurl --version`.

pub mod hurl;
pub mod version;

/// `gurl --help`: the subcommands. Each one has its own `--help`.
pub fn help() -> String {
    format!(
        "{}\nA GUI for HURL that also runs from the terminal.\n\n\
Usage: gurl                      open the app\n       \
gurl cli [OPTIONS] [FILE]... run HURL files exactly like `hurl` (+ workspace gurl.env variables)\n       \
gurl --version              versions of gURL, HURL and the native libraries\n\n\
Run `gurl cli --help` for HURL's options.\n",
        version::short()
    )
}
