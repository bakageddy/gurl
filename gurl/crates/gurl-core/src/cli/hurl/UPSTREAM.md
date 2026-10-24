# Forked from HURL

The files in this directory (except `workspace.rs` and this note) are copied from the `hurl` binary
sources of HURL **8.0.1** (`vendor/hurl/packages/hurl/src/{main.rs,run.rs,cli/**}`), Copyright (C)
Orange, Apache License 2.0. They are not part of the published `hurl` library, which is why they are
vendored here rather than depended on.

Local changes, kept as small as possible so a HURL bump is a `diff -r` against the upstream directory:

- `main.rs` → `entry.rs`: `fn main() -> ExitCode` became `pub fn main(args: &[OsString]) -> i32`;
  `HurlRun` is `pub`; `--version`/`-V` is answered before clap so the report isn't prefixed with the command name.
- `crate::cli::…` → `crate::cli::hurl::…`, `crate::runner` → `hurl::runner`.
- `options/args.rs`: the command is named `gurl cli` and parses the given `args` instead of
  `std::env::args_os()`; two gURL options (`--workspace`, `--no-workspace`) are registered.
- `options/mod.rs`: `get_version()` delegates to `crate::cli::version`; `CliOptions` has the two
  extra fields.
- `run.rs`: variables come from `workspace::variables_for` (gurl.env chain first, command line on top).
- `options/secret.rs`: one `&` dropped in a `format!` argument (newer clippy lint, `-D warnings`).
