//! Everything `gurl` does from a terminal: `gurl cli` (HURL's CLI), `gurl --version`.

pub mod fmt;
pub mod hurl;
pub mod version;

/// `gurl --help`: the subcommands. Each one has its own `--help`.
pub fn help() -> String {
    format!(
        "{}\nA GUI for HURL that also runs from the terminal.\n\n\
Usage: gurl                      open the app\n       \
gurl cli [OPTIONS] [FILE]... run HURL files exactly like `hurl` (+ workspace gurl.env variables)\n       \
gurl fmt [OPTIONS] [FILE]... format / check / export HURL files like `hurlfmt` (+ directories)\n       \
gurl --version              versions of gURL, HURL and the native libraries\n\n\
Run `gurl cli --help` or `gurl fmt --help` for their options.\n",
        version::short()
    )
}
