//! Previews of what HURL's `jsonpath` and `xpath` queries return for a response body, used by the
//! app's filter box so that what the user sees is what an assert would compute.
//!
//! JSONPath is evaluated by hurl's own RFC 9535 implementation (compiled in from the vendored
//! sources, see `lib.rs`) with hurl's result mapping (`runner/filter/jsonpath.rs`): no node → none,
//! one node → that value, several → a list. XPath goes through libxml2 like hurl's `runner/xpath.rs`.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryKind {
    Jsonpath,
    Xpath,
}

/// What a query produced. `kind` tells the UI how to render `value` and which predicates make sense.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOutcome {
    /// `none` (JSONPath matched nothing), `single`, `list` (JSONPath), `number`, `bool`, `string`,
    /// `nodeset` (XPath), or `error`.
    pub kind: &'static str,
    pub value: Option<serde_json::Value>,
    /// Elements of a JSONPath list / nodes of an XPath node set.
    pub count: usize,
    pub error: Option<String>,
}

impl QueryOutcome {
    fn error(msg: impl Into<String>) -> Self {
        QueryOutcome {
            kind: "error",
            value: None,
            count: 0,
            error: Some(msg.into()),
        }
    }
}

/// Lists longer than this are cut in the preview (`count` keeps the real length).
const MAX_LIST: usize = 1000;

pub fn query(kind: QueryKind, body: &[u8], content_type: Option<&str>, expr: &str) -> QueryOutcome {
    match kind {
        QueryKind::Jsonpath => query_json(body, expr),
        QueryKind::Xpath => query_xpath(body, content_type, expr),
    }
}

pub fn query_json(body: &[u8], expr: &str) -> QueryOutcome {
    let text = match std::str::from_utf8(body) {
        Ok(t) => t,
        Err(_) => return QueryOutcome::error("body is not UTF-8 text"),
    };
    let json: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => return QueryOutcome::error(format!("body is not JSON: {e}")),
    };
    let q = match crate::jsonpath::parse(expr) {
        Ok(q) => q,
        Err(_) => return QueryOutcome::error("invalid JSONPath expression"),
    };
    let mut results = q.eval(&json);
    match results.len() {
        0 => QueryOutcome {
            kind: "none",
            value: None,
            count: 0,
            error: None,
        },
        1 => QueryOutcome {
            kind: "single",
            value: Some(results.remove(0)),
            count: 1,
            error: None,
        },
        n => {
            results.truncate(MAX_LIST);
            QueryOutcome {
                kind: "list",
                value: Some(serde_json::Value::Array(results)),
                count: n,
                error: None,
            }
        }
    }
}

pub fn query_xpath(body: &[u8], content_type: Option<&str>, expr: &str) -> QueryOutcome {
    let text = match std::str::from_utf8(body) {
        Ok(t) => t,
        Err(_) => return QueryOutcome::error("body is not UTF-8 text"),
    };
    let html = content_type.is_some_and(|c| c.to_ascii_lowercase().contains("html"));
    let doc = match parse_document(text, html) {
        Some(d) => d,
        None => {
            return QueryOutcome::error(if html {
                "body is not HTML"
            } else {
                "body is not XML"
            });
        }
    };
    let context = match libxml::xpath::Context::new(&doc) {
        Ok(c) => c,
        Err(_) => return QueryOutcome::error("cannot create XPath context"),
    };
    // libxml2 prints warnings and errors to stderr otherwise.
    unsafe {
        libxml::bindings::xmlSetGenericErrorFunc(ptr::null_mut(), Some(silent_error_func));
    }
    if !html {
        register_namespaces(&doc, &context);
    }
    let object = match context.evaluate(expr) {
        Ok(o) => o,
        Err(_) => return QueryOutcome::error("invalid XPath expression"),
    };
    // SAFETY: `object.ptr` is a valid xmlXPathObject owned by `object` for its lifetime.
    let raw = unsafe { &*object.ptr };
    match raw.type_ {
        libxml::bindings::xmlXPathObjectType_XPATH_NUMBER => QueryOutcome {
            kind: "number",
            value: serde_json::Number::from_f64(raw.floatval).map(serde_json::Value::Number),
            count: 1,
            error: None,
        },
        libxml::bindings::xmlXPathObjectType_XPATH_BOOLEAN => QueryOutcome {
            kind: "bool",
            value: Some(serde_json::Value::Bool(raw.boolval != 0)),
            count: 1,
            error: None,
        },
        libxml::bindings::xmlXPathObjectType_XPATH_STRING => {
            // SAFETY: a string XPath object always carries a NUL-terminated `stringval`.
            let s = unsafe { CStr::from_ptr(raw.stringval as *const c_char) }
                .to_string_lossy()
                .into_owned();
            QueryOutcome {
                kind: "string",
                value: Some(serde_json::Value::String(s)),
                count: 1,
                error: None,
            }
        }
        libxml::bindings::xmlXPathObjectType_XPATH_NODESET => {
            let n = object.get_number_of_nodes();
            QueryOutcome {
                kind: "nodeset",
                value: Some(serde_json::Value::from(n)),
                count: n,
                error: None,
            }
        }
        _ => QueryOutcome::error("unsupported XPath result"),
    }
}

/// Parses with the same libxml2 calls and options hurl uses (recover + no error output).
fn parse_document(input: &str, html: bool) -> Option<libxml::tree::Document> {
    let len = i32::try_from(input.len()).ok()?;
    let encoding = CString::new("utf-8").ok()?;
    let options = 1 + 32; // HTML_PARSE_RECOVER | HTML_PARSE_NOERROR
    // SAFETY: pointers come from live Rust buffers that outlive the calls; libxml2 copies the data.
    let ptr = unsafe {
        let p = input.as_ptr() as *const c_char;
        if html {
            libxml::bindings::htmlReadMemory(p, len, ptr::null(), encoding.as_ptr(), options)
        } else {
            libxml::bindings::xmlReadMemory(p, len, ptr::null(), encoding.as_ptr(), options)
        }
    };
    if ptr.is_null() {
        return None;
    }
    let doc = libxml::tree::Document::new_ptr(ptr);
    doc.get_root_element()?;
    Some(doc)
}

unsafe extern "C" fn silent_error_func(
    _ctx: *mut std::os::raw::c_void,
    _msg: *const c_char,
    _: ...
) {
}

/// Same convention as hurl: every prefixed namespace is registered, the first default one as `_`.
fn register_namespaces(doc: &libxml::tree::Document, context: &libxml::xpath::Context) {
    let Some(root) = doc.get_root_element() else {
        return;
    };
    let mut default_registered = false;
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        for ns in node.get_namespace_declarations() {
            let prefix = ns.get_prefix();
            if prefix.is_empty() {
                if !default_registered {
                    let _ = context.register_namespace("_", &ns.get_href());
                    default_registered = true;
                }
            } else {
                let _ = context.register_namespace(&prefix, &ns.get_href());
            }
        }
        stack.extend(node.get_child_elements());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const BODY: &[u8] =
        br#"{"items":[{"id":1,"name":"a"},{"id":2,"name":"b"}],"ok":true,"n":1.50}"#;

    #[test]
    fn jsonpath_follows_hurl_result_mapping() {
        let one = query_json(BODY, "$.items[0].id");
        assert_eq!((one.kind, one.value), ("single", Some(json!(1))));
        let many = query_json(BODY, "$.items[*].name");
        assert_eq!(
            (many.kind, many.count, many.value),
            ("list", 2, Some(json!(["a", "b"])))
        );
        assert_eq!(query_json(BODY, "$.missing").kind, "none");
        assert_eq!(
            query_json(BODY, "$.items[?@.id > 1].name").value,
            Some(json!("b"))
        );
        // serde_json's arbitrary_precision (enabled by hurl) keeps the number text as written.
        assert_eq!(query_json(BODY, "$.n").value.unwrap().to_string(), "1.50");
        assert!(query_json(BODY, "$[").error.is_some());
        assert!(query_json(b"not json", "$").error.is_some());
    }

    #[test]
    fn xpath_returns_typed_results() {
        let xml = b"<root><a id=\"x\">one</a><a>two</a></root>";
        let count = query_xpath(xml, Some("application/xml"), "count(//a)");
        assert_eq!((count.kind, count.value), ("number", Some(json!(2.0))));
        let s = query_xpath(xml, None, "string(//a[1])");
        assert_eq!((s.kind, s.value), ("string", Some(json!("one"))));
        let b = query_xpath(xml, None, "boolean(//a/@id)");
        assert_eq!((b.kind, b.value), ("bool", Some(json!(true))));
        let nodes = query_xpath(xml, None, "//a");
        assert_eq!((nodes.kind, nodes.count), ("nodeset", 2));
        assert!(query_xpath(xml, None, "//a[").error.is_some());

        let html = b"<html><body><h1>Hi</h1><p>x</p><p>y</p></body></html>";
        let h = query_xpath(html, Some("text/html; charset=utf-8"), "string(//h1)");
        assert_eq!(h.value, Some(json!("Hi")));
        assert_eq!(
            query_xpath(html, Some("text/html"), "count(//p)").value,
            Some(json!(2.0))
        );
    }
}
