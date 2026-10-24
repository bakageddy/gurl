//! `gurl fmt`: `hurlfmt` (format, `--check`, export to JSON/HTML, import from curl) with one extra:
//! directories and `--workspace` expand to every `.hurl` file below them, so `gurl fmt --check .`
//! lints a workspace and `gurl fmt --in-place .` formats it.
//!
//! The work is done by the `hurlfmt` library; only the option parsing is ours, because hurlfmt's
//! parser reads `std::env::args_os()` and its `matches` module is private. Options, output and exit
//! codes (0 ok, 1 error, 2 invalid input, 3 lint issue) are hurlfmt's.

use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command};
use hurl_core::input::{Input, InputKind};
use hurl_core::text;
use hurlfmt::cli::Logger;
use hurlfmt::cli::options::{InputFormat, OutputFormat};
use hurlfmt::command;
use hurlfmt::command::check::CheckError;
use hurlfmt::command::export::ExportError;
use hurlfmt::command::format::FormatError;

use crate::workspace::collect_hurl_files;

const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_LINT_ISSUE: i32 = 3;

struct Options {
    check: bool,
    color: Option<bool>,
    in_place: bool,
    input_files: Vec<Input>,
    input_format: InputFormat,
    output_file: Option<PathBuf>,
    output_format: OutputFormat,
    standalone: bool,
}

fn command() -> Command {
    Command::new("gurl fmt")
        .version(crate::cli::version::short())
        .disable_colored_help(true)
        .about("Format Hurl files (hurlfmt)")
        .arg(
            Arg::new("input_files")
                .value_name("FILES")
                .help("Hurl files, or directories to format every .hurl file below (default: standard input)")
                .num_args(1..),
        )
        .arg(Arg::new("check").long("check").help("Run in 'check' mode").action(ArgAction::SetTrue))
        .arg(
            Arg::new("color")
                .long("color")
                .help("Colorize output")
                .action(ArgAction::SetTrue)
                .conflicts_with("no_color")
                .conflicts_with("in_place"),
        )
        .arg(
            Arg::new("in_place")
                .long("in-place")
                .help("Modify files in place")
                .action(ArgAction::SetTrue)
                .conflicts_with("output"),
        )
        .arg(
            Arg::new("input_format")
                .long("in")
                .value_name("FORMAT")
                .value_parser(["hurl", "curl"])
                .default_value("hurl")
                .help("Specify input format: hurl or curl"),
        )
        .arg(Arg::new("no_color").long("no-color").help("Do not colorize output").action(ArgAction::SetTrue))
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Write output to FILE instead of stdout"),
        )
        .arg(
            Arg::new("output_format")
                .long("out")
                .value_name("FORMAT")
                .value_parser(["hurl", "json", "html"])
                .default_value("hurl")
                .help("Specify output format: hurl, json or html"),
        )
        .arg(
            Arg::new("standalone")
                .long("standalone")
                .help("Standalone HTML")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("workspace")
                .long("workspace")
                .value_name("DIR")
                .help("Format/check every .hurl file below DIR (gURL extra)")
                .help_heading("gURL options"),
        )
}

/// Executes `gurl fmt` with `args` (everything after `gurl fmt`) and returns the exit code.
pub fn main(args: &[OsString]) -> i32 {
    text::init_crate_colored();
    let argv = std::iter::once(OsString::from("gurl fmt")).chain(args.iter().cloned());
    let mut cmd = command();
    let matches = match cmd.try_get_matches_from_mut(argv) {
        Ok(m) => m,
        Err(e) => {
            use clap::error::ErrorKind::{DisplayHelp, DisplayVersion};
            if matches!(e.kind(), DisplayHelp | DisplayVersion) {
                print!("{e}");
                return EXIT_OK;
            }
            eprintln!("{e}");
            return EXIT_ERROR;
        }
    };
    let opts = match options(&matches) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_ERROR;
        }
    };
    if opts.input_files.is_empty() {
        eprint!("{}", cmd.render_help());
        return EXIT_ERROR;
    }

    let color = opts.color.unwrap_or_else(|| io::stdout().is_terminal());
    let logger = Logger::new(color);
    if opts.check {
        check(&opts.input_files, opts.output_file, &logger)
    } else if opts.in_place {
        format_in_place(&opts.input_files, &logger)
    } else {
        export(&opts, &logger, color)
    }
}

fn options(m: &ArgMatches) -> Result<Options, String> {
    let mut input_files = Vec::new();
    if let Some(dir) = m.get_one::<String>("workspace") {
        expand_dir(Path::new(dir), &mut input_files)?;
    }
    for name in m.get_many::<String>("input_files").into_iter().flatten() {
        let path = Path::new(name);
        if path.is_dir() {
            expand_dir(path, &mut input_files)?;
        } else if path.is_file() {
            input_files.push(Input::new(name));
        } else {
            return Err(format!(
                "hurl: cannot access '{name}': No such file or directory"
            ));
        }
    }
    if input_files.is_empty()
        && m.get_one::<String>("workspace").is_none()
        && !io::stdin().is_terminal()
    {
        let input = Input::from_stdin().map_err(|e| format!("cannot read standard input: {e}"))?;
        input_files.push(input);
    }
    let in_place = m.get_flag("in_place");
    if in_place
        && input_files
            .iter()
            .any(|i| matches!(i.kind(), InputKind::Stdin(_)))
    {
        return Err("Standard input can not be formatted in place!".to_string());
    }
    let color = if m.get_flag("color") {
        Some(true)
    } else if m.get_flag("no_color") || in_place {
        Some(false)
    } else {
        None
    };
    let input_format = match m.get_one::<String>("input_format").map(String::as_str) {
        Some("curl") => InputFormat::Curl,
        _ => InputFormat::Hurl,
    };
    let output_format = match m.get_one::<String>("output_format").map(String::as_str) {
        Some("json") => OutputFormat::Json,
        Some("html") => OutputFormat::Html,
        _ => OutputFormat::Hurl,
    };
    if in_place && output_format != OutputFormat::Hurl {
        return Err("You can use --in-place only hurl format!".to_string());
    }
    Ok(Options {
        check: m.get_flag("check"),
        color,
        in_place,
        input_files,
        input_format,
        output_file: m.get_one::<String>("output").map(PathBuf::from),
        output_format,
        standalone: m.get_flag("standalone"),
    })
}

/// Every `.hurl` file below `dir` (gURL extra: directories as inputs).
fn expand_dir(dir: &Path, input_files: &mut Vec<Input>) -> Result<(), String> {
    let files = collect_hurl_files(dir, dir, true).map_err(|e| e.to_string())?;
    if files.is_empty() {
        return Err(format!("no .hurl file below {}", dir.display()));
    }
    input_files.extend(files.into_iter().map(|p| Input::new(&p.to_string_lossy())));
    Ok(())
}

fn check(input_files: &[Input], output_file: Option<PathBuf>, logger: &Logger) -> i32 {
    let errors = command::check::run(input_files);
    if errors.is_empty() {
        return EXIT_OK;
    }
    let mut count = 0;
    let mut invalid_input = false;
    let mut output_all = String::new();
    for e in &errors {
        match e {
            CheckError::IO { filename, message } => {
                logger.error(&format!(
                    "Input file {filename} can not be read - {message}"
                ));
                invalid_input = true;
            }
            CheckError::Parse {
                content,
                input_file,
                error,
            } => {
                logger.error_parsing(content, input_file, error);
                invalid_input = true;
            }
            CheckError::Unformatted(filename) => {
                output_all.push_str(&format!("would reformat: {filename}\n"));
                count += 1;
            }
        }
    }
    if count > 0 {
        output_all.push_str(&format!(
            "{count} file{} would be reformatted",
            if count > 1 { "s" } else { "" }
        ));
    }
    if let Err(code) = write_output(&output_all, output_file, logger) {
        return code;
    }
    if invalid_input {
        EXIT_INVALID_INPUT
    } else {
        EXIT_LINT_ISSUE
    }
}

fn format_in_place(input_files: &[Input], logger: &Logger) -> i32 {
    let paths: Vec<PathBuf> = input_files
        .iter()
        .filter_map(|i| match i.kind() {
            InputKind::File(p) => Some(p.clone()),
            InputKind::Stdin(_) => None,
        })
        .collect();
    let errors = command::format::run(&paths);
    if errors.is_empty() {
        return EXIT_OK;
    }
    for e in &errors {
        match e {
            FormatError::IO { filename, message } => {
                logger.error(&format!(
                    "Input file {filename} can not be read - {message}"
                ));
            }
            FormatError::Parse {
                content,
                input_file,
                error,
            } => {
                logger.error_parsing(content, input_file, error);
            }
        }
    }
    EXIT_INVALID_INPUT
}

fn export(opts: &Options, logger: &Logger, color: bool) -> i32 {
    let mut error = false;
    let mut output_all = String::new();
    let results = command::export::run(
        &opts.input_files,
        &opts.input_format,
        &opts.output_format,
        opts.standalone,
        color,
    );
    for result in &results {
        match result {
            Ok(output) => output_all.push_str(output),
            Err(e) => {
                error = true;
                match e {
                    ExportError::IO { filename, message } => {
                        logger.error(&format!(
                            "Input file {filename} can not be read - {message}"
                        ));
                    }
                    ExportError::Parse {
                        content,
                        input_file,
                        error,
                    } => {
                        logger.error_parsing(content, input_file, error);
                    }
                    ExportError::Curl(s) => logger.error(&format!("error curl {s}")),
                }
            }
        }
    }
    if let Err(code) = write_output(&output_all, opts.output_file.clone(), logger) {
        return code;
    }
    if error { EXIT_INVALID_INPUT } else { EXIT_OK }
}

fn write_output(content: &str, filename: Option<PathBuf>, logger: &Logger) -> Result<(), i32> {
    let content = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{content}\n")
    };
    let result = match filename {
        None => io::stdout().lock().write_all(content.as_bytes()),
        Some(path) => {
            std::fs::File::create(&path).and_then(|mut f| f.write_all(content.as_bytes()))
        }
    };
    result.map_err(|e| {
        logger.error(&format!("Issue writing output: {e}"));
        EXIT_ERROR
    })
}

/// Formats one HURL document in memory (the app's "Format" action). Errors are the parse error,
/// rendered with its position, as hurlfmt prints it.
pub fn format_text(text: &str) -> Result<String, String> {
    use hurl_core::error::{DisplaySourceError, OutputFormat};
    match hurl_core::parser::parse_hurl_file(text) {
        Ok(file) => Ok(hurlfmt::linter::lint_hurl_file(&file)),
        Err(e) => Err(e.render("", text, None, OutputFormat::Plain)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_text_and_reports_parse_errors() {
        let messy = "GET   http://x\nHTTP   200\n[Asserts]\nstatus   ==   200\n";
        let formatted = format_text(messy).unwrap();
        assert_eq!(
            formatted,
            "GET http://x\nHTTP 200\n[Asserts]\nstatus == 200\n"
        );
        assert_eq!(format_text(&formatted).unwrap(), formatted);
        assert!(format_text("GET http://x\nHTTP 200\n[Asserts\n").is_err());
    }

    #[test]
    fn directories_expand_to_hurl_files() {
        let root = std::env::temp_dir().join(format!("gurl-fmt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("a.hurl"), "GET  http://x\n").unwrap();
        std::fs::write(root.join("sub").join("b.hurl"), "GET http://y\n").unwrap();
        std::fs::write(root.join("notes.txt"), "x").unwrap();
        let m = command()
            .try_get_matches_from(["gurl fmt", "--check", root.to_str().unwrap()])
            .unwrap();
        let opts = options(&m).unwrap();
        assert_eq!(opts.input_files.len(), 2);
        assert!(opts.check);
        let _ = std::fs::remove_dir_all(&root);
    }
}
