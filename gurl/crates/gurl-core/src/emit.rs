//! [`HurlDoc`] -> canonical HURL text.

use crate::model::*;

fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn kv_lines(out: &mut String, items: &[Kv], sep: &str) {
    for kv in items {
        out.push_str(&format!("{}{sep} {}\n", kv.name, kv.value));
    }
}

fn section(out: &mut String, name: &str) {
    out.push_str(&format!("[{name}]\n"));
}

fn query(q: &QueryM, filters: &[FilterM]) -> String {
    let mut s = join(&[&q.kind, &q.args]);
    for f in filters {
        s.push(' ');
        s.push_str(&join(&[&f.name, &f.args]));
    }
    s
}

fn body(out: &mut String, b: &BodyM) {
    let fenced = |out: &mut String, lang: &str, text: &str| {
        out.push_str("```");
        out.push_str(lang);
        out.push('\n');
        out.push_str(text);
        if !text.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
    };
    match b.kind.as_str() {
        "multiline" => fenced(out, "", &b.text),
        "multiline-json" => fenced(out, "json", &b.text),
        "multiline-xml" => fenced(out, "xml", &b.text),
        "multiline-raw" => fenced(out, "raw", &b.text),
        "graphql" => fenced(out, "graphql", &b.text),
        "base64" => out.push_str(&format!("base64,{};\n", b.text)),
        "hex" => out.push_str(&format!("hex,{};\n", b.text)),
        "file" => out.push_str(&format!("file,{};\n", b.text)),
        // json / xml / text: written as-is. Oneline text containing a newline can't stay
        // oneline, so promote it.
        "text" if b.text.contains('\n') => fenced(out, "", &b.text),
        _ => {
            out.push_str(&b.text);
            if !b.text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
}

/// One `[Captures]` line, without the newline.
pub fn capture_line(c: &CaptureM) -> String {
    let mut line = format!("{}: {}", c.name, query(&c.query, &c.filters));
    if c.redacted {
        line.push_str(" redact");
    }
    line
}

/// One `[Asserts]` line, without the newline.
pub fn assert_line(a: &AssertM) -> String {
    let not = if a.not { "not" } else { "" };
    join(&[&query(&a.query, &a.filters), not, &a.func, &a.value])
}

pub fn emit(doc: &HurlDoc) -> String {
    let mut out = String::new();
    for (i, e) in doc.entries.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for c in &e.comments {
            if c.is_empty() {
                out.push_str("#\n");
            } else {
                out.push_str(&format!("# {c}\n"));
            }
        }
        let r = &e.request;
        out.push_str(&format!("{} {}\n", r.method, r.url));
        kv_lines(&mut out, &r.headers, ":");
        if !r.query.is_empty() {
            section(&mut out, "Query");
            kv_lines(&mut out, &r.query, ":");
        }
        if !r.form.is_empty() {
            section(&mut out, "Form");
            kv_lines(&mut out, &r.form, ":");
        }
        if !r.multipart.is_empty() {
            section(&mut out, "Multipart");
            for p in &r.multipart {
                if p.kind == "file" {
                    let mut v = format!("file,{};", p.value);
                    if let Some(ct) = p.content_type.as_ref().filter(|c| !c.is_empty()) {
                        v.push(' ');
                        v.push_str(ct);
                    }
                    out.push_str(&format!("{}: {v}\n", p.name));
                } else {
                    out.push_str(&format!("{}: {}\n", p.name, p.value));
                }
            }
        }
        if !r.cookies.is_empty() {
            section(&mut out, "Cookies");
            kv_lines(&mut out, &r.cookies, ":");
        }
        if let Some(a) = &r.basic_auth {
            section(&mut out, "BasicAuth");
            kv_lines(&mut out, std::slice::from_ref(a), ":");
        }
        if !r.options.is_empty() {
            section(&mut out, "Options");
            kv_lines(&mut out, &r.options, ":");
        }
        if let Some(b) = &r.body {
            body(&mut out, b);
        }
        if let Some(resp) = &e.response {
            out.push_str(&format!("{} {}\n", resp.version, resp.status));
            kv_lines(&mut out, &resp.headers, ":");
            if !resp.captures.is_empty() {
                section(&mut out, "Captures");
                for c in &resp.captures {
                    out.push_str(&capture_line(c));
                    out.push('\n');
                }
            }
            if !resp.asserts.is_empty() {
                section(&mut out, "Asserts");
                for a in &resp.asserts {
                    out.push_str(&assert_line(a));
                    out.push('\n');
                }
            }
            if let Some(b) = &resp.body {
                body(&mut out, b);
            }
        }
    }
    if !doc.trailing_comments.is_empty() {
        if !doc.entries.is_empty() {
            out.push('\n');
        }
        for c in &doc.trailing_comments {
            out.push_str(&format!("# {c}\n"));
        }
    }
    out
}
