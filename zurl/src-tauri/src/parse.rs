//! HURL text -> [`HurlDoc`] using `hurl_core`'s parser.

use crate::emit;
use crate::model::*;
use hurl_core::ast::{self, Bytes, MultilineStringKind, SectionValue, SourceInfo};
use hurl_core::error::DisplaySourceError;
use hurl_core::parser::parse_hurl_file;
use hurl_core::types::ToSource;

/// Text of `info` (1-based line/column, columns counted in chars; end exclusive).
fn slice(lines: &[&str], info: SourceInfo) -> String {
    let (sl, sc, el, ec) = (
        info.start.line,
        info.start.column,
        info.end.line,
        info.end.column,
    );
    if sl == 0 || sl > lines.len() {
        return String::new();
    }
    let mut out = String::new();
    for ln in sl..=el.min(lines.len()) {
        let line = lines[ln - 1];
        let from = if ln == sl { sc.saturating_sub(1) } else { 0 };
        let to = if ln == el { ec.saturating_sub(1) } else { usize::MAX };
        let piece: String = line
            .chars()
            .skip(from)
            .take(to.saturating_sub(from))
            .collect();
        out.push_str(&piece);
        if ln != el {
            out.push('\n');
        }
    }
    out
}

fn split_first(s: &str) -> (String, String) {
    let s = s.trim();
    match s.find(char::is_whitespace) {
        Some(i) => (s[..i].to_string(), s[i..].trim().to_string()),
        None => (s.to_string(), String::new()),
    }
}

struct Ctx<'a> {
    lines: Vec<&'a str>,
    comments_seen: usize,
    comments_kept: usize,
}

impl Ctx<'_> {
    fn term(&mut self, t: &ast::LineTerminator) {
        if t.comment.is_some() {
            self.comments_seen += 1;
        }
    }
    fn terms(&mut self, ts: &[ast::LineTerminator]) {
        for t in ts {
            self.term(t);
        }
    }

    fn kv(&mut self, kv: &ast::KeyValue) -> Kv {
        self.terms(&kv.line_terminators);
        self.term(&kv.line_terminator0);
        Kv {
            name: kv.key.to_source().as_str().to_string(),
            value: kv.value.to_source().as_str().to_string(),
        }
    }

    fn query(&self, q: &ast::Query) -> QueryM {
        let (kind, args) = split_first(&slice(&self.lines, q.source_info));
        QueryM { kind, args }
    }

    fn filters(&self, fs: &[(ast::Whitespace, ast::Filter)]) -> Vec<FilterM> {
        fs.iter()
            .map(|(_, f)| {
                let (name, args) = split_first(&slice(&self.lines, f.source_info));
                FilterM { name, args }
            })
            .collect()
    }

    fn body(&mut self, b: &ast::Body) -> BodyM {
        self.terms(&b.line_terminators);
        self.term(&b.line_terminator0);
        match &b.value {
            Bytes::Json(v) => BodyM {
                kind: "json".into(),
                text: v.to_source().as_str().to_string(),
            },
            Bytes::Xml(s) => BodyM {
                kind: "xml".into(),
                text: s.clone(),
            },
            Bytes::OnelineString(t) => BodyM {
                kind: "text".into(),
                text: t.to_source().as_str().to_string(),
            },
            Bytes::MultilineString(m) => {
                let kind = match &m.kind {
                    MultilineStringKind::Text(_) => "multiline",
                    MultilineStringKind::Json(_) => "multiline-json",
                    MultilineStringKind::Xml(_) => "multiline-xml",
                    MultilineStringKind::Raw(_) => "multiline-raw",
                    MultilineStringKind::GraphQl(_) => "graphql",
                };
                BodyM {
                    kind: kind.into(),
                    text: m.kind.to_source().as_str().to_string(),
                }
            }
            Bytes::Base64(b) => BodyM {
                kind: "base64".into(),
                text: b.source.as_str().to_string(),
            },
            Bytes::Hex(h) => BodyM {
                kind: "hex".into(),
                text: h.source.as_str().to_string(),
            },
            Bytes::File(f) => BodyM {
                kind: "file".into(),
                text: f.filename.to_source().as_str().to_string(),
            },
        }
    }

    fn assert(&mut self, a: &ast::Assert) -> AssertM {
        self.terms(&a.line_terminators);
        self.term(&a.line_terminator0);
        let text = slice(&self.lines, a.predicate.predicate_func.source_info);
        let (func, value) = split_first(&text);
        AssertM {
            query: self.query(&a.query),
            filters: self.filters(&a.filters),
            not: a.predicate.not,
            func,
            value,
        }
    }

    fn capture(&mut self, c: &ast::Capture) -> CaptureM {
        self.terms(&c.line_terminators);
        self.term(&c.line_terminator0);
        CaptureM {
            name: c.name.to_source().as_str().to_string(),
            query: self.query(&c.query),
            filters: self.filters(&c.filters),
            redacted: c.redacted,
        }
    }

    fn request(&mut self, r: &ast::Request) -> (RequestM, Vec<String>) {
        let mut comments = Vec::new();
        for t in &r.line_terminators {
            if let Some(c) = &t.comment {
                comments.push(c.value.trim_start().to_string());
                self.comments_seen += 1;
                self.comments_kept += 1;
            }
        }
        self.term(&r.line_terminator0);
        let mut m = RequestM {
            method: r.method.to_string(),
            url: r.url.to_source().as_str().to_string(),
            ..Default::default()
        };
        for h in &r.headers {
            m.headers.push(self.kv(h));
        }
        for s in &r.sections {
            self.terms(&s.line_terminators);
            self.term(&s.line_terminator0);
            match &s.value {
                SectionValue::QueryParams(ps, _) => {
                    for p in ps {
                        m.query.push(self.kv(p));
                    }
                }
                SectionValue::FormParams(ps, _) => {
                    for p in ps {
                        m.form.push(self.kv(p));
                    }
                }
                SectionValue::MultipartFormData(ps, _) => {
                    for p in ps {
                        match p {
                            ast::MultipartParam::Param(kv) => {
                                let kv = self.kv(kv);
                                m.multipart.push(MultipartM {
                                    name: kv.name,
                                    kind: "param".into(),
                                    value: kv.value,
                                    content_type: None,
                                });
                            }
                            ast::MultipartParam::FilenameParam(f) => {
                                self.terms(&f.line_terminators);
                                self.term(&f.line_terminator0);
                                m.multipart.push(MultipartM {
                                    name: f.key.to_source().as_str().to_string(),
                                    kind: "file".into(),
                                    value: f.value.filename.to_source().as_str().to_string(),
                                    content_type: f
                                        .value
                                        .content_type
                                        .as_ref()
                                        .map(|t| t.to_source().as_str().to_string()),
                                });
                            }
                        }
                    }
                }
                SectionValue::Cookies(cs) => {
                    for c in cs {
                        self.terms(&c.line_terminators);
                        self.term(&c.line_terminator0);
                        m.cookies.push(Kv {
                            name: c.name.to_source().as_str().to_string(),
                            value: c.value.to_source().as_str().to_string(),
                        });
                    }
                }
                SectionValue::BasicAuth(kv) => {
                    m.basic_auth = kv.as_ref().map(|k| self.kv(k));
                }
                SectionValue::Options(opts) => {
                    for o in opts {
                        self.terms(&o.line_terminators);
                        self.term(&o.line_terminator0);
                        // `Display` renders `identifier: value`.
                        let text = o.kind.to_string();
                        let name = o.kind.identifier().to_string();
                        let value = text
                            .strip_prefix(&format!("{name}: "))
                            .unwrap_or("")
                            .to_string();
                        m.options.push(Kv { name, value });
                    }
                }
                SectionValue::Captures(_) | SectionValue::Asserts(_) => {}
            }
        }
        if let Some(b) = &r.body {
            m.body = Some(self.body(b));
        }
        (m, comments)
    }

    fn response(&mut self, r: &ast::Response) -> ResponseM {
        self.terms(&r.line_terminators);
        self.term(&r.line_terminator0);
        let mut m = ResponseM {
            version: r.version.value.to_string(),
            status: r.status.value.to_string(),
            ..Default::default()
        };
        for h in &r.headers {
            m.headers.push(self.kv(h));
        }
        for s in &r.sections {
            self.terms(&s.line_terminators);
            self.term(&s.line_terminator0);
            match &s.value {
                SectionValue::Captures(cs) => {
                    for c in cs {
                        let c = self.capture(c);
                        m.captures.push(c);
                    }
                }
                SectionValue::Asserts(asserts) => {
                    for a in asserts {
                        let a = self.assert(a);
                        m.asserts.push(a);
                    }
                }
                _ => {}
            }
        }
        if let Some(b) = &r.body {
            m.body = Some(self.body(b));
        }
        m
    }
}

fn convert(text: &str) -> Result<(HurlDoc, usize), Diagnostic> {
    let file = parse_hurl_file(text).map_err(|e| {
        let info = e.source_info();
        Diagnostic {
            line: info.start.line,
            column: info.start.column,
            end_line: info.end.line,
            end_column: info.end.column,
            message: e.description(),
        }
    })?;
    let mut cx = Ctx {
        lines: text.lines().collect(),
        comments_seen: 0,
        comments_kept: 0,
    };
    let mut doc = HurlDoc::default();
    for entry in &file.entries {
        let (request, comments) = cx.request(&entry.request);
        let response = entry.response.as_ref().map(|r| cx.response(r));
        doc.entries.push(EntryM {
            comments,
            request,
            response,
        });
    }
    for t in &file.line_terminators {
        if let Some(c) = &t.comment {
            doc.trailing_comments.push(c.value.trim_start().to_string());
            cx.comments_seen += 1;
            cx.comments_kept += 1;
        }
    }
    let dropped = cx.comments_seen - cx.comments_kept;
    Ok((doc, dropped))
}

pub fn parse(text: &str) -> ParseOutcome {
    match convert(text) {
        Err(d) => ParseOutcome {
            model: None,
            diagnostics: vec![d],
            lossless: false,
            dropped_comments: 0,
        },
        Ok((doc, dropped)) => {
            let emitted = emit::emit(&doc);
            let lossless = match convert(&emitted) {
                Ok((again, _)) => again == doc,
                Err(_) => false,
            };
            ParseOutcome {
                model: Some(doc),
                diagnostics: vec![],
                lossless,
                dropped_comments: dropped,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# login
POST https://example.org/login?a=1
Content-Type: application/json
[Query]
q: {{term}}
[Options]
retry: 2
insecure: true
```json
{"user": "{{user}}"}
```
HTTP 200
[Captures]
token: jsonpath "$.token" nth 0
[Asserts]
status == 200
header "X-A" not contains "b"
jsonpath "$.items" count > 3
body matches /ab+/
duration < 500
jsonpath "$.id" exists
"#;

    #[test]
    fn parses_and_roundtrips() {
        let out = parse(SAMPLE);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let doc = out.model.unwrap();
        assert!(out.lossless, "emit: {}", emit::emit(&doc));
        let e = &doc.entries[0];
        assert_eq!(e.comments, vec!["login"]);
        assert_eq!(e.request.method, "POST");
        assert_eq!(e.request.options.len(), 2);
        assert_eq!(e.request.body.as_ref().unwrap().kind, "multiline-json");
        let r = e.response.as_ref().unwrap();
        assert_eq!(r.status, "200");
        assert_eq!(r.captures[0].filters[0].name, "nth");
        assert_eq!(r.asserts[0].func, "==");
        assert!(r.asserts[1].not);
        assert_eq!(r.asserts[1].func, "contains");
        assert_eq!(r.asserts[2].filters[0].name, "count");
        assert_eq!(r.asserts[5].func, "exists");
    }

    #[test]
    fn reports_syntax_errors() {
        let out = parse("GET\n");
        assert!(out.model.is_none());
        assert_eq!(out.diagnostics.len(), 1);
    }

    #[test]
    fn fixture_corpus_roundtrips() {
        let text = include_str!("../fixtures/all.hurl");
        let out = parse(text);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let doc = out.model.unwrap();
        assert_eq!(doc.entries.len(), 9);
        assert!(out.lossless, "not lossless; emit:\n{}", emit::emit(&doc));
        assert_eq!(out.dropped_comments, 0);
    }
}
