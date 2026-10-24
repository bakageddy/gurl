//! Structured, serde-friendly view of a HURL file used by the visual builder.
//!
//! Leaf values (URLs, header values, predicate values, filter args...) are kept as
//! raw HURL source text, so templates (`{{var}}`) and quoting survive untouched.
//! Only the grammar skeleton (entries, sections, queries, predicates) is typed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HurlDoc {
    pub entries: Vec<EntryM>,
    /// Comment lines found after the last entry.
    pub trailing_comments: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EntryM {
    /// Comment lines (without the leading `#`) directly above the request.
    pub comments: Vec<String>,
    pub request: RequestM,
    pub response: Option<ResponseM>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Kv {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestM {
    pub method: String,
    pub url: String,
    pub headers: Vec<Kv>,
    pub query: Vec<Kv>,
    pub form: Vec<Kv>,
    pub multipart: Vec<MultipartM>,
    pub cookies: Vec<Kv>,
    pub basic_auth: Option<Kv>,
    pub options: Vec<Kv>,
    pub body: Option<BodyM>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MultipartM {
    pub name: String,
    /// `param` (plain text) or `file`.
    pub kind: String,
    pub value: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BodyM {
    /// json | xml | text | multiline | multiline-json | multiline-xml | multiline-raw |
    /// graphql | base64 | hex | file
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResponseM {
    /// `HTTP`, `HTTP/1.0`, `HTTP/1.1`, `HTTP/2` or `HTTP/3`.
    pub version: String,
    /// A status code or `*`.
    pub status: String,
    pub headers: Vec<Kv>,
    pub captures: Vec<CaptureM>,
    pub asserts: Vec<AssertM>,
    pub body: Option<BodyM>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QueryM {
    pub kind: String,
    pub args: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FilterM {
    pub name: String,
    pub args: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaptureM {
    pub name: String,
    pub query: QueryM,
    pub filters: Vec<FilterM>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AssertM {
    pub query: QueryM,
    pub filters: Vec<FilterM>,
    pub not: bool,
    pub func: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseOutcome {
    pub model: Option<HurlDoc>,
    pub diagnostics: Vec<Diagnostic>,
    /// `true` when `emit(model)` parses back to an identical model.
    pub lossless: bool,
    /// Comments that the builder can't represent and would drop on rewrite.
    pub dropped_comments: usize,
}
