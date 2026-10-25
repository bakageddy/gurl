//! gURL extras for `gurl cli`: variables from the workspace's `gurl.env` files and headers from its
//! `gurl.headers` files.
//!
//! Variables, lowest to highest: `gurl.env` chain (root → file directory) < hurl config file <
//! `HURL_*` environment < `--variables-file` < `--variable` / `--secret`. The last four are already
//! merged in [`CliOptions::variables`] / [`CliOptions::secrets`] by the option parser, so this module
//! only has to seed the set and let those override.
//!
//! Headers: the `gurl.headers` chain (rendered with the file's variables) comes before `--header`,
//! so a command-line header wins over the workspace, which wins over the file.

use std::path::{Path, PathBuf};

use hurl::runner::VariableSet;
use hurl_core::input::{Input, InputKind};

use super::error::CliError;
use super::options::CliOptions;
use crate::hurl::{parse_value, render_template};
use crate::workspace::{ENV_FILE, HEADERS_FILE, effective_headers, effective_vars};

/// The options to run `input` with: the given ones plus the workspace's `gurl.headers`, rendered
/// with `variables`, ahead of the command line's `--header`s.
pub fn options_for(
    input: &Input,
    options: &CliOptions,
    variables: &VariableSet,
) -> Result<CliOptions, CliError> {
    let Some((root, dir)) = scope_of(input, options) else {
        return Ok(options.clone());
    };
    let Ok(headers) = effective_headers(&root, &dir) else {
        return Ok(options.clone());
    };
    let mut rendered = Vec::with_capacity(headers.len() + options.headers.len());
    for h in headers {
        let value = render_template(&h.value, variables)
            .map_err(|e| CliError::InvalidOption(format!("{HEADERS_FILE}: {}: {e}", h.name)))?;
        rendered.push(format!("{}: {value}", h.name));
    }
    rendered.extend(options.headers.iter().cloned());
    let mut out = options.clone();
    out.headers = rendered;
    Ok(out)
}

/// Builds the variable set for one input file: workspace variables first, then everything the
/// command line defined on top.
pub fn variables_for(input: &Input, options: &CliOptions) -> VariableSet {
    let mut variables = VariableSet::new();
    if let Some((root, dir)) = scope_of(input, options)
        && let Ok(vars) = effective_vars(&root, &dir)
    {
        for v in vars {
            if v.secret {
                variables.insert_secret(v.name, v.value);
            } else {
                variables.insert(v.name, parse_value(&v.value));
            }
        }
    }
    for (name, value) in &options.variables {
        variables.insert(name.clone(), value.clone());
    }
    for (name, value) in &options.secrets {
        variables.insert_secret(name.clone(), value.clone());
    }
    variables
}

/// The `(workspace root, file directory)` pair whose `gurl.env` files apply to `input`, or `None`
/// when no workspace applies (`--no-workspace`, stdin without `--workspace`, no `gurl.env` found).
fn scope_of(input: &Input, options: &CliOptions) -> Option<(PathBuf, PathBuf)> {
    if options.no_workspace {
        return None;
    }
    let dir = match input.kind() {
        InputKind::File(path) => absolute_parent(path)?,
        InputKind::Stdin(_) => std::env::current_dir().ok()?,
    };
    let root = match &options.workspace {
        Some(root) => std::fs::canonicalize(root).unwrap_or_else(|_| root.clone()),
        None => discover_root(&dir)?,
    };
    if !dir.starts_with(&root) {
        return None;
    }
    Some((root, dir))
}

fn absolute_parent(path: &Path) -> Option<PathBuf> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = match parent {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };
    Some(std::fs::canonicalize(&dir).unwrap_or(dir))
}

/// Walks up from `dir` and returns the topmost ancestor (inclusive) that holds a `gurl.env` or a
/// `gurl.headers`.
fn discover_root(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .filter(|d| d.join(ENV_FILE).is_file() || d.join(HEADERS_FILE).is_file())
        .last()
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("gurl-cli-ws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::canonicalize(p).unwrap()
    }

    #[test]
    fn discovers_topmost_env_and_cli_variables_win() {
        let root = tmp("discover");
        let sub = root.join("api").join("v2");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(
            root.join(ENV_FILE),
            "host=root\nport=8080\n# @secret\ntoken=abc\n",
        )
        .unwrap();
        std::fs::write(sub.join(ENV_FILE), "host=v2\n").unwrap();
        std::fs::write(root.join("api").join("x.hurl"), "GET http://x\n").unwrap();
        let file = sub.join("a.hurl");
        std::fs::write(&file, "GET http://x\n").unwrap();

        let mut options = CliOptions::default();
        options.variables.insert("port".into(), parse_value("9999"));
        let vars = variables_for(&Input::new(&file.to_string_lossy()), &options);
        assert_eq!(vars.get("host").unwrap().value().to_string(), "v2");
        assert_eq!(vars.get("port").unwrap().value().to_string(), "9999");
        assert!(vars.secrets().contains(&"abc".to_string()));

        options.no_workspace = true;
        let vars = variables_for(&Input::new(&file.to_string_lossy()), &options);
        assert!(vars.get("host").is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn workspace_headers_are_rendered_and_sit_below_cli_headers() {
        let root = tmp("headers");
        let sub = root.join("api");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(root.join(ENV_FILE), "token=abc\n").unwrap();
        std::fs::write(
            root.join(HEADERS_FILE),
            "Authorization: Bearer {{token}}\nX-Trace: root\n",
        )
        .unwrap();
        std::fs::write(sub.join(HEADERS_FILE), "X-Trace: api\n").unwrap();
        let file = sub.join("a.hurl");
        std::fs::write(&file, "GET http://x\n").unwrap();
        let input = Input::new(&file.to_string_lossy());

        let mut options = CliOptions::default();
        options.headers.push("X-Cli: 1".to_string());
        let variables = variables_for(&input, &options);
        let per_file = options_for(&input, &options, &variables).unwrap();
        assert_eq!(
            per_file.headers,
            ["Authorization: Bearer abc", "X-Trace: api", "X-Cli: 1"]
        );

        std::fs::write(root.join(HEADERS_FILE), "X-Key: {{nope}}\n").unwrap();
        assert!(options_for(&input, &options, &variables).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn no_env_means_no_workspace() {
        let root = tmp("none");
        let file = root.join("a.hurl");
        std::fs::write(&file, "GET http://x\n").unwrap();
        let options = CliOptions::default();
        assert!(scope_of(&Input::new(&file.to_string_lossy()), &options).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }
}
