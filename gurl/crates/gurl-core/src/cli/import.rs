//! `gurl import <curl|postman|opencollection> <FILE|DIR|-> [--out DIR] [--one-file]`: the app's
//! importers from the terminal. Without `--out` the HURL text is printed (handy for
//! `pbpaste | gurl import curl -`); with it, files are written below DIR and listed.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};

use crate::import::{self, ImportKind, ImportOptions, ImportSource};

fn command() -> Command {
    Command::new("gurl import")
        .version(crate::cli::version::short())
        .disable_colored_help(true)
        .about("Convert cURL commands, Postman collections or OpenCollection YAML to HURL files")
        .arg(
            Arg::new("kind")
                .value_name("FORMAT")
                .value_parser(["curl", "postman", "opencollection"])
                .required(true)
                .help("Input format"),
        )
        .arg(
            Arg::new("input")
                .value_name("FILE")
                .required(true)
                .help("A file, a directory (OpenCollection layout), or - for standard input"),
        )
        .arg(
            Arg::new("out")
                .long("out")
                .short('o')
                .value_name("DIR")
                .help("Write the .hurl files (and gurl.env) below DIR instead of printing them"),
        )
        .arg(
            Arg::new("one_file")
                .long("one-file")
                .help("One .hurl per folder (requests concatenated) instead of one per request")
                .action(ArgAction::SetTrue),
        )
}

pub fn main(args: &[OsString]) -> i32 {
    let argv = std::iter::once(OsString::from("gurl import")).chain(args.iter().cloned());
    let matches = match command().try_get_matches_from(argv) {
        Ok(m) => m,
        Err(e) => {
            use clap::error::ErrorKind::{DisplayHelp, DisplayVersion};
            if matches!(e.kind(), DisplayHelp | DisplayVersion) {
                print!("{e}");
                return 0;
            }
            eprintln!("{e}");
            return 1;
        }
    };
    let kind = match matches.get_one::<String>("kind").map(String::as_str) {
        Some("curl") => ImportKind::Curl,
        Some("postman") => ImportKind::Postman,
        _ => ImportKind::Opencollection,
    };
    let input = matches
        .get_one::<String>("input")
        .cloned()
        .unwrap_or_default();
    let source = if input == "-" {
        let mut text = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut text) {
            eprintln!("error: cannot read standard input: {e}");
            return 2;
        }
        ImportSource {
            text: Some(text),
            path: None,
        }
    } else {
        ImportSource {
            text: None,
            path: Some(input),
        }
    };
    let options = ImportOptions {
        one_file: matches.get_flag("one_file"),
    };
    let result = match import::import(kind, &source, &options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    for w in &result.warnings {
        eprintln!("warning: {w}");
    }
    match matches.get_one::<String>("out") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            let root: &Path = &dir;
            match import::write(&result, root, &dir) {
                Ok(paths) => {
                    for p in paths {
                        println!("{}", p.display());
                    }
                    0
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    1
                }
            }
        }
        None => {
            for (i, f) in result.files.iter().enumerate() {
                if result.files.len() > 1 {
                    if i > 0 {
                        println!();
                    }
                    println!("# ---- {}", f.rel_path);
                }
                print!("{}", f.text);
            }
            for e in &result.envs {
                println!(
                    "\n# ---- {}{}",
                    if e.rel_dir.is_empty() {
                        String::new()
                    } else {
                        format!("{}/", e.rel_dir)
                    },
                    e.file_name
                );
                print!("{}", crate::workspace::emit_env(&e.vars));
            }
            0
        }
    }
}
