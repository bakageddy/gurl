//! Lossless edits of HURL text: adding asserts and captures to one entry without re-emitting the
//! file, so comments and formatting survive (the app's "add assert from this value").

use hurl_core::ast::{Entry, LineTerminator, MultipartParam, Response, Section, SectionValue};
use hurl_core::parser::parse_hurl_file;
use serde::Serialize;
use thiserror::Error;

use crate::emit::{assert_line, capture_line};
use crate::model::{AssertM, CaptureM};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EditError {
    #[error("{0}")]
    Parse(String),
    #[error("entry {0} does not exist")]
    EntryOutOfRange(usize),
    #[error("already present: {0}")]
    Duplicate(String),
    #[error("nothing to add")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub text: String,
    /// 1-based line of the first inserted line.
    pub first_line: usize,
    pub added: usize,
}

/// Adds `asserts` to the `[Asserts]` section and `captures` to the `[Captures]` section of entry
/// `entry` (1-based, as in HURL's reports), creating the sections (and an `HTTP *` response) as
/// needed. Lines identical to one already in the section are skipped; if all are, that's an error.
pub fn add_response_items(
    text: &str,
    entry: usize,
    asserts: &[AssertM],
    captures: &[CaptureM],
) -> Result<Edit, EditError> {
    let file = parse_hurl_file(text).map_err(|e| {
        use hurl_core::error::{DisplaySourceError, OutputFormat};
        EditError::Parse(e.render("", text, None, OutputFormat::Plain))
    })?;
    let ast_entry = entry
        .checked_sub(1)
        .and_then(|i| file.entries.get(i))
        .ok_or(EditError::EntryOutOfRange(entry))?;

    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let source_lines: Vec<&str> = text.lines().collect();
    let mut first_line = None;
    let mut added = 0;
    let mut duplicate = None;

    let mut plan: Vec<(&'static str, Vec<String>)> = Vec::new();
    if !asserts.is_empty() {
        plan.push(("Asserts", asserts.iter().map(assert_line).collect()));
    }
    if !captures.is_empty() {
        plan.push(("Captures", captures.iter().map(capture_line).collect()));
    }
    if plan.is_empty() {
        return Err(EditError::Empty);
    }
    // Where each section's items would go, computed on the original text; the insertion lower in
    // the file happens first so the upper line number stays valid.
    let mut placements: Vec<(usize, Vec<String>)> = plan
        .into_iter()
        .map(|(name, new_lines)| {
            let (after_line, existing, has_section) = placement(ast_entry, name, &source_lines);
            let fresh: Vec<String> = new_lines
                .into_iter()
                .filter(|l| {
                    let dup = existing.iter().any(|e| e.trim() == l.trim());
                    if dup {
                        duplicate.get_or_insert_with(|| l.clone());
                    }
                    !dup
                })
                .collect();
            let mut block = Vec::new();
            if !fresh.is_empty() && !has_section {
                if ast_entry.response.is_none()
                    && name == first_section_for_missing_response(asserts, captures)
                {
                    block.push("HTTP *".to_string());
                }
                block.push(format!("[{name}]"));
            }
            block.extend(fresh);
            (after_line, block)
        })
        .collect();
    if placements.iter().all(|(_, block)| block.is_empty()) {
        return Err(EditError::Duplicate(duplicate.unwrap_or_default()));
    }
    // Bottom-most insertion first.
    placements.sort_by_key(|p| std::cmp::Reverse(p.0));
    // When both sections are created on a response-less entry they share the insertion line and
    // `HTTP *` must come only once, before the upper one (Captures).
    let mut http_emitted = false;
    for (after_line, block) in placements.iter_mut() {
        if block.is_empty() {
            continue;
        }
        if block.first().is_some_and(|l| l == "HTTP *") {
            if http_emitted {
                block.remove(0);
            } else {
                http_emitted = true;
            }
        }
        // Make sure the line we append after ends with a newline.
        if *after_line > 0 {
            let prev = &mut lines[*after_line - 1];
            if !prev.ends_with('\n') {
                prev.push_str(eol);
            }
        }
        let rendered: Vec<String> = block.iter().map(|l| format!("{l}{eol}")).collect();
        lines.splice(*after_line..*after_line, rendered);
        let first_item = *after_line
            + 1
            + block
                .iter()
                .take_while(|l| l.starts_with('[') || l.starts_with("HTTP"))
                .count();
        first_line = Some(first_item);
        added += block
            .iter()
            .filter(|l| !l.starts_with('[') && !l.starts_with("HTTP"))
            .count();
    }
    Ok(Edit {
        text: lines.concat(),
        first_line: first_line.unwrap_or(1),
        added,
    })
}

/// For an entry without a response both sections may be created; `HTTP *` goes with whichever is
/// inserted first in the file (Captures precede Asserts) — but since insertion happens bottom-up,
/// we attach it to the section that will end up on top.
fn first_section_for_missing_response(asserts: &[AssertM], captures: &[CaptureM]) -> &'static str {
    if !captures.is_empty() {
        "Captures"
    } else if !asserts.is_empty() {
        "Asserts"
    } else {
        ""
    }
}

/// `(line to insert after (0 = top of file), existing item lines, section exists)`.
fn placement(entry: &Entry, name: &str, source_lines: &[&str]) -> (usize, Vec<String>, bool) {
    match &entry.response {
        Some(resp) => {
            if let Some(section) = resp.sections.iter().find(|s| s.identifier() == name) {
                let existing = section_item_lines(section, source_lines);
                let last = section_end_line(section);
                return (last, existing, true);
            }
            // New section: after the last existing section, else after the headers / status line.
            let after = resp
                .sections
                .iter()
                .map(section_end_line)
                .max()
                .or_else(|| resp.headers.last().map(|h| line_of(&h.line_terminator0)))
                .unwrap_or_else(|| response_status_line(resp));
            (after, Vec::new(), false)
        }
        None => (request_end_line(entry), Vec::new(), false),
    }
}

fn line_of(lt: &LineTerminator) -> usize {
    lt.newline.source_info.start.line
}

fn response_status_line(resp: &Response) -> usize {
    line_of(&resp.line_terminator0)
}

fn section_end_line(section: &Section) -> usize {
    let items: Vec<usize> = match &section.value {
        SectionValue::Asserts(a) => a.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        SectionValue::Captures(c) => c.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        SectionValue::QueryParams(kv, _) | SectionValue::FormParams(kv, _) => {
            kv.iter().map(|x| line_of(&x.line_terminator0)).collect()
        }
        SectionValue::BasicAuth(kv) => kv.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        SectionValue::MultipartFormData(p, _) => p
            .iter()
            .map(|x| match x {
                MultipartParam::Param(kv) => line_of(&kv.line_terminator0),
                MultipartParam::FilenameParam(f) => line_of(&f.line_terminator0),
            })
            .collect(),
        SectionValue::Cookies(c) => c.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        SectionValue::Options(o) => o.iter().map(|x| line_of(&x.line_terminator0)).collect(),
    };
    items
        .into_iter()
        .max()
        .unwrap_or_else(|| line_of(&section.line_terminator0))
}

/// Source text of a section's items, one per item (for duplicate detection); a trailing comment is
/// dropped so `status == 200 # ok` still matches `status == 200`.
fn section_item_lines(section: &Section, source_lines: &[&str]) -> Vec<String> {
    let lines: Vec<usize> = match &section.value {
        SectionValue::Asserts(a) => a.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        SectionValue::Captures(c) => c.iter().map(|x| line_of(&x.line_terminator0)).collect(),
        _ => Vec::new(),
    };
    lines
        .into_iter()
        .filter_map(|n| source_lines.get(n - 1))
        .map(|l| l.split(" #").next().unwrap_or(l).trim().to_string())
        .collect()
}

fn request_end_line(entry: &Entry) -> usize {
    let req = &entry.request;
    if let Some(body) = &req.body {
        return line_of(&body.line_terminator0);
    }
    req.sections
        .iter()
        .map(section_end_line)
        .max()
        .or_else(|| req.headers.last().map(|h| line_of(&h.line_terminator0)))
        .unwrap_or_else(|| line_of(&req.line_terminator0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FilterM, QueryM};

    fn assert(kind: &str, args: &str, func: &str, value: &str) -> AssertM {
        AssertM {
            query: QueryM {
                kind: kind.into(),
                args: args.into(),
            },
            filters: vec![],
            not: false,
            func: func.into(),
            value: value.into(),
        }
    }

    fn capture(name: &str) -> CaptureM {
        CaptureM {
            name: name.into(),
            query: QueryM {
                kind: "jsonpath".into(),
                args: "\"$.id\"".into(),
            },
            filters: vec![FilterM {
                name: "toInt".into(),
                args: String::new(),
            }],
            redacted: false,
        }
    }

    #[test]
    fn appends_to_existing_section_keeping_comments() {
        let text = "# login\nGET http://x\nHTTP 200\n[Asserts]\nstatus == 200 # fine\n\n# second\nGET http://y\n";
        let e =
            add_response_items(text, 1, &[assert("jsonpath", "\"$.a\"", "==", "1")], &[]).unwrap();
        assert_eq!(
            e.text,
            "# login\nGET http://x\nHTTP 200\n[Asserts]\nstatus == 200 # fine\njsonpath \"$.a\" == 1\n\n# second\nGET http://y\n"
        );
        assert_eq!((e.first_line, e.added), (6, 1));
    }

    #[test]
    fn creates_section_before_body_and_after_headers() {
        let text = "GET http://x\nHTTP 200\nContent-Type: text/plain\n```\nhello\n```\n";
        let e =
            add_response_items(text, 1, &[assert("header", "\"X\"", "exists", "")], &[]).unwrap();
        assert_eq!(
            e.text,
            "GET http://x\nHTTP 200\nContent-Type: text/plain\n[Asserts]\nheader \"X\" exists\n```\nhello\n```\n"
        );
        assert_eq!(e.first_line, 5);
    }

    #[test]
    fn creates_response_when_missing() {
        let text = "GET http://x\n[Options]\nretry: 2\n\nGET http://y\n";
        let e = add_response_items(
            text,
            1,
            &[assert("status", "", "==", "200")],
            &[capture("id")],
        )
        .unwrap();
        assert_eq!(
            e.text,
            "GET http://x\n[Options]\nretry: 2\nHTTP *\n[Captures]\nid: jsonpath \"$.id\" toInt\n[Asserts]\nstatus == 200\n\nGET http://y\n"
        );
        // Round-trips: still parses, with two entries.
        assert_eq!(parse_hurl_file(&e.text).unwrap().entries.len(), 2);
    }

    #[test]
    fn crlf_and_missing_final_newline() {
        let text = "GET http://x\r\nHTTP 200";
        let e = add_response_items(text, 1, &[assert("status", "", "==", "200")], &[]).unwrap();
        assert_eq!(
            e.text,
            "GET http://x\r\nHTTP 200\r\n[Asserts]\r\nstatus == 200\r\n"
        );
    }

    #[test]
    fn duplicates_and_bad_entries_are_errors() {
        let text = "GET http://x\nHTTP 200\n[Asserts]\nstatus == 200\n";
        let dup = add_response_items(text, 1, &[assert("status", "", "==", "200")], &[]);
        assert_eq!(dup, Err(EditError::Duplicate("status == 200".into())));
        assert_eq!(
            add_response_items(text, 2, &[assert("status", "", "==", "200")], &[]),
            Err(EditError::EntryOutOfRange(2))
        );
        assert!(matches!(
            add_response_items("GET\n", 1, &[], &[]),
            Err(EditError::Parse(_))
        ));
        assert_eq!(add_response_items(text, 1, &[], &[]), Err(EditError::Empty));
    }
}
