//! Postman Collection Format v2.1 → folders of HURL files. Requests map onto the builder model;
//! scripts and examples have no HURL equivalent and are reported as warnings.

use serde::Deserialize;
use serde_json::Value;

use super::{
    Folder, ImportError, ImportOptions, ImportResult, Request, layout, path_params_to_templates,
};
use crate::model::{BodyM, EntryM, Kv, MultipartM, RequestM, ResponseM};
use crate::workspace::EnvVar;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Collection {
    info: Info,
    item: Vec<Item>,
    variable: Vec<Variable>,
    auth: Option<Auth>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Info {
    name: String,
    schema: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Item {
    name: String,
    item: Option<Vec<Item>>,
    request: Option<RequestDef>,
    event: Vec<Value>,
    auth: Option<Auth>,
    variable: Vec<Variable>,
    description: Option<Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RequestDef {
    Url(String),
    Full(Box<RequestFull>),
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RequestFull {
    method: String,
    header: Vec<Header>,
    url: Option<Url>,
    body: Option<Body>,
    auth: Option<Auth>,
    description: Option<Value>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Header {
    key: String,
    value: String,
    disabled: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Url {
    Raw(String),
    Full(UrlFull),
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct UrlFull {
    raw: String,
    protocol: Option<String>,
    host: Option<Value>,
    port: Option<String>,
    path: Option<Value>,
    query: Vec<QueryParam>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct QueryParam {
    key: Option<String>,
    value: Option<String>,
    disabled: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Body {
    mode: String,
    raw: Option<String>,
    urlencoded: Vec<Header>,
    formdata: Vec<FormParam>,
    file: Option<FileRef>,
    graphql: Option<GraphQl>,
    options: Option<BodyOptions>,
    disabled: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct FormParam {
    key: String,
    value: Option<String>,
    src: Option<Value>,
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    disabled: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct FileRef {
    src: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct GraphQl {
    query: String,
    variables: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct BodyOptions {
    raw: Option<RawOptions>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawOptions {
    language: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Auth {
    #[serde(rename = "type")]
    kind: String,
    basic: Vec<AuthParam>,
    bearer: Vec<AuthParam>,
    apikey: Vec<AuthParam>,
    awsv4: Vec<AuthParam>,
    digest: Vec<AuthParam>,
    ntlm: Vec<AuthParam>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct AuthParam {
    key: String,
    value: Value,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Variable {
    key: String,
    value: Value,
    disabled: bool,
}

pub fn import(text: &str, options: &ImportOptions) -> Result<ImportResult, ImportError> {
    let col: Collection = serde_json::from_str(text)
        .map_err(|e| ImportError::Invalid(format!("not a Postman collection: {e}")))?;
    if col.info.name.is_empty() && col.item.is_empty() {
        return Err(ImportError::Invalid(
            "not a Postman collection (no info/item)".to_string(),
        ));
    }
    let mut out = ImportResult::default();
    if !col.info.schema.contains("v2.1") && !col.info.schema.is_empty() {
        out.warnings.push(format!(
            "collection schema is {}; v2.1 is what this importer knows",
            col.info.schema
        ));
    }
    let mut root = Folder::new(if col.info.name.is_empty() {
        "postman"
    } else {
        &col.info.name
    });
    root.vars = variables(&col.variable);
    let mut ctx = Ctx {
        warnings: Vec::new(),
        scripts: 0,
    };
    convert_items(&col.item, col.auth.as_ref(), &mut root, &mut ctx);
    if ctx.scripts > 0 {
        ctx.warnings.push(format!(
            "{} pre-request/test script(s) have no HURL equivalent and were dropped",
            ctx.scripts
        ));
    }
    out.warnings.extend(ctx.warnings);
    layout(&root, options, &mut out);
    Ok(out)
}

struct Ctx {
    warnings: Vec<String>,
    scripts: usize,
}

fn variables(vars: &[Variable]) -> Vec<EnvVar> {
    vars.iter()
        .filter(|v| !v.disabled && !v.key.is_empty())
        .map(|v| EnvVar {
            name: v.key.clone(),
            value: value_str(&v.value),
            secret: false,
        })
        .collect()
}

fn value_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn convert_items(
    items: &[Item],
    inherited_auth: Option<&Auth>,
    folder: &mut Folder,
    ctx: &mut Ctx,
) {
    for item in items {
        ctx.scripts += item.event.len();
        let auth = item.auth.as_ref().or(inherited_auth);
        if let Some(children) = &item.item {
            let mut sub = Folder::new(&item.name);
            sub.vars = variables(&item.variable);
            convert_items(children, auth, &mut sub, ctx);
            folder.folders.push(sub);
        } else if let Some(def) = &item.request {
            let entry = convert_request(item, def, auth, ctx);
            folder.requests.push(Request {
                name: item.name.clone(),
                entry,
            });
        }
    }
}

fn convert_request(item: &Item, def: &RequestDef, auth: Option<&Auth>, ctx: &mut Ctx) -> EntryM {
    let mut req = RequestM::default();
    let mut comments = Vec::new();
    if !item.name.is_empty() {
        comments.push(item.name.clone());
    }
    match def {
        RequestDef::Url(u) => {
            req.method = "GET".into();
            req.url = path_params_to_templates(u);
        }
        RequestDef::Full(full) => {
            req.method = if full.method.is_empty() {
                "GET".into()
            } else {
                full.method.to_uppercase()
            };
            if let Some(d) = full.description.as_ref().and_then(description) {
                comments.extend(d.lines().map(str::to_string));
            }
            match &full.url {
                Some(Url::Raw(u)) => req.url = path_params_to_templates(u),
                Some(Url::Full(u)) => {
                    let raw = if u.raw.is_empty() {
                        assemble_url(u)
                    } else {
                        u.raw.clone()
                    };
                    // Query params are kept in the URL as Postman wrote them, except disabled ones.
                    let base = raw.split('?').next().unwrap_or(&raw).to_string();
                    req.url = path_params_to_templates(&base);
                    for q in &u.query {
                        if q.disabled {
                            continue;
                        }
                        let Some(k) = &q.key else { continue };
                        req.query.push(Kv {
                            name: k.clone(),
                            value: q.value.clone().unwrap_or_default(),
                        });
                    }
                    if u.query.is_empty() && raw.contains('?') {
                        req.url = path_params_to_templates(&raw);
                    }
                }
                None => {}
            }
            for h in &full.header {
                if !h.disabled && !h.key.is_empty() {
                    req.headers.push(Kv {
                        name: h.key.clone(),
                        value: h.value.clone(),
                    });
                }
            }
            if let Some(b) = &full.body
                && !b.disabled
            {
                convert_body(b, &mut req, ctx);
            }
            apply_auth(full.auth.as_ref().or(auth), &mut req, ctx);
            if full.auth.is_none() && auth.is_none() {
                // nothing
            }
        }
    }
    if matches!(def, RequestDef::Url(_)) {
        apply_auth(auth, &mut req, ctx);
    }
    EntryM {
        comments,
        request: req,
        response: Some(ResponseM {
            version: "HTTP".into(),
            status: "*".into(),
            ..Default::default()
        }),
    }
}

fn description(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Object(o) => o
            .get("content")
            .and_then(|c| c.as_str())
            .map(|s| s.trim().to_string()),
        _ => None,
    }
}

fn assemble_url(u: &UrlFull) -> String {
    let host = match &u.host {
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join("."),
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let path = match &u.path {
        Some(Value::Array(parts)) => parts
            .iter()
            .map(|p| match p {
                Value::String(s) => s.clone(),
                Value::Object(o) => o
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("/"),
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let mut url = String::new();
    if let Some(p) = &u.protocol {
        url.push_str(p);
        url.push_str("://");
    }
    url.push_str(&host);
    if let Some(port) = &u.port {
        url.push(':');
        url.push_str(port);
    }
    if !path.is_empty() {
        url.push('/');
        url.push_str(&path);
    }
    url
}

fn convert_body(b: &Body, req: &mut RequestM, ctx: &mut Ctx) {
    match b.mode.as_str() {
        "raw" => {
            let raw = b.raw.clone().unwrap_or_default();
            if raw.trim().is_empty() {
                return;
            }
            let lang = b
                .options
                .as_ref()
                .and_then(|o| o.raw.as_ref())
                .and_then(|r| r.language.clone());
            let ct = req
                .headers
                .iter()
                .find(|h| h.name.eq_ignore_ascii_case("content-type"))
                .map(|h| h.value.to_ascii_lowercase());
            let kind = match lang.as_deref() {
                Some("json") => "json",
                Some("xml") => "xml",
                _ if ct.as_deref().is_some_and(|c| c.contains("json")) => "json",
                _ if ct.as_deref().is_some_and(|c| c.contains("xml")) => "xml",
                _ => "text",
            };
            let kind = match kind {
                "json" if serde_json::from_str::<Value>(&raw).is_err() => {
                    ctx.warnings.push("a JSON body isn't valid JSON (variables inside strings?) and was kept as text".to_string());
                    "text"
                }
                k => k,
            };
            if kind == "json" {
                req.headers
                    .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
            }
            req.body = Some(BodyM {
                kind: kind.into(),
                text: raw,
            });
        }
        "urlencoded" => {
            for p in &b.urlencoded {
                if !p.disabled && !p.key.is_empty() {
                    req.form.push(Kv {
                        name: p.key.clone(),
                        value: p.value.clone(),
                    });
                }
            }
            req.headers
                .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
        }
        "formdata" => {
            for p in &b.formdata {
                if p.disabled || p.key.is_empty() {
                    continue;
                }
                if p.kind == "file" {
                    let src = match &p.src {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Array(a)) => {
                            a.first().and_then(|v| v.as_str()).unwrap_or("").to_string()
                        }
                        _ => String::new(),
                    };
                    let file = src.rsplit(['/', '\\']).next().unwrap_or(&src).to_string();
                    ctx.warnings.push(format!(
                        "multipart file {file} must be copied next to the scenario"
                    ));
                    req.multipart.push(MultipartM {
                        name: p.key.clone(),
                        kind: "file".into(),
                        value: file,
                        content_type: p.content_type.clone(),
                    });
                } else {
                    req.multipart.push(MultipartM {
                        name: p.key.clone(),
                        kind: "param".into(),
                        value: p.value.clone().unwrap_or_default(),
                        content_type: None,
                    });
                }
            }
            req.headers
                .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
        }
        "file" => {
            let src = b
                .file
                .as_ref()
                .and_then(|f| f.src.clone())
                .unwrap_or_default();
            let file = src.rsplit(['/', '\\']).next().unwrap_or(&src).to_string();
            ctx.warnings.push(format!(
                "body file {file} must be copied next to the scenario"
            ));
            req.body = Some(BodyM {
                kind: "file".into(),
                text: file,
            });
        }
        "graphql" => {
            if let Some(g) = &b.graphql {
                let mut text = g.query.trim_end().to_string();
                if let Some(v) = g
                    .variables
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty() && *v != "{}")
                {
                    text.push_str("\n\nvariables ");
                    text.push_str(v);
                }
                req.body = Some(BodyM {
                    kind: "graphql".into(),
                    text,
                });
            }
        }
        "" | "none" => {}
        other => ctx
            .warnings
            .push(format!("body mode {other} is not supported")),
    }
}

fn apply_auth(auth: Option<&Auth>, req: &mut RequestM, ctx: &mut Ctx) {
    let Some(auth) = auth else { return };
    let get = |params: &[AuthParam], key: &str| {
        params
            .iter()
            .find(|p| p.key == key)
            .map(|p| value_str(&p.value))
            .unwrap_or_default()
    };
    match auth.kind.as_str() {
        "basic" => {
            req.basic_auth = Some(Kv {
                name: get(&auth.basic, "username"),
                value: get(&auth.basic, "password"),
            });
        }
        "bearer" => req.headers.push(Kv {
            name: "Authorization".into(),
            value: format!("Bearer {}", get(&auth.bearer, "token")),
        }),
        "apikey" => {
            let key = get(&auth.apikey, "key");
            let value = get(&auth.apikey, "value");
            if get(&auth.apikey, "in") == "query" {
                req.query.push(Kv { name: key, value });
            } else {
                req.headers.push(Kv { name: key, value });
            }
        }
        "awsv4" => {
            let region = get(&auth.awsv4, "region");
            let service = get(&auth.awsv4, "service");
            req.options.push(Kv {
                name: "aws-sigv4".into(),
                value: format!("aws:amz:{region}:{service}"),
            });
            req.basic_auth = Some(Kv {
                name: get(&auth.awsv4, "accessKey"),
                value: get(&auth.awsv4, "secretKey"),
            });
        }
        "digest" => {
            req.options.push(Kv {
                name: "digest".into(),
                value: "true".into(),
            });
            req.basic_auth = Some(Kv {
                name: get(&auth.digest, "username"),
                value: get(&auth.digest, "password"),
            });
        }
        "ntlm" => {
            req.options.push(Kv {
                name: "ntlm".into(),
                value: "true".into(),
            });
            req.basic_auth = Some(Kv {
                name: get(&auth.ntlm, "username"),
                value: get(&auth.ntlm, "password"),
            });
        }
        "noauth" | "inherit" | "" => {}
        other => ctx.warnings.push(format!(
            "auth type {other} is not supported; add the header by hand"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "info": { "name": "Shop API", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json" },
      "variable": [{ "key": "host", "value": "https://api.example.com" }],
      "auth": { "type": "bearer", "bearer": [{ "key": "token", "value": "{{token}}" }] },
      "item": [
        { "name": "Users", "item": [
          { "name": "Get user", "request": { "method": "GET",
            "url": { "raw": "{{host}}/users/:id?full=true", "host": ["{{host}}"], "path": ["users", ":id"],
                     "query": [{ "key": "full", "value": "true" }, { "key": "x", "value": "1", "disabled": true }] },
            "header": [{ "key": "Accept", "value": "application/json" }] } },
          { "name": "Create user", "event": [{ "listen": "test", "script": {} }],
            "request": { "method": "POST", "url": "{{host}}/users",
              "header": [{ "key": "Content-Type", "value": "application/json" }],
              "body": { "mode": "raw", "raw": "{\"name\": \"x\"}", "options": { "raw": { "language": "json" } } },
              "auth": { "type": "basic", "basic": [{ "key": "username", "value": "u" }, { "key": "password", "value": "p" }] } } }
        ] },
        { "name": "Upload", "request": { "method": "POST", "url": "{{host}}/files",
          "body": { "mode": "formdata", "formdata": [{ "key": "f", "type": "file", "src": "/tmp/a.png" }, { "key": "k", "value": "v", "type": "text" }] } } },
        { "name": "Login form", "request": { "method": "POST", "url": "{{host}}/login",
          "body": { "mode": "urlencoded", "urlencoded": [{ "key": "u", "value": "a" }, { "key": "p", "value": "b" }] } } }
      ]
    }"#;

    #[test]
    fn converts_folders_requests_auth_and_bodies() {
        let out = import(SAMPLE, &ImportOptions::default()).unwrap();
        let paths: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "upload.hurl",
                "login-form.hurl",
                "users/get-user.hurl",
                "users/create-user.hurl"
            ]
        );
        assert_eq!(out.envs[0].vars[0].name, "host");

        let get = &out.files[2].text;
        assert_eq!(
            get,
            "# Get user\nGET {{host}}/users/{{id}}\nAccept: application/json\nAuthorization: Bearer {{token}}\n[Query]\nfull: true\nHTTP *\n"
        );
        let create = &out.files[3].text;
        assert!(
            create.contains("POST {{host}}/users\n[BasicAuth]\nu: p\n{\"name\": \"x\"}\n"),
            "{create}"
        );
        let upload = &out.files[0].text;
        assert!(
            upload.contains("[Multipart]\nf: file,a.png;\nk: v\n"),
            "{upload}"
        );
        let login = &out.files[1].text;
        assert!(login.contains("[Form]\nu: a\np: b\n"), "{login}");
        assert!(out.warnings.iter().any(|w| w.contains("script")));
        assert!(out.warnings.iter().any(|w| w.contains("a.png")));
    }

    #[test]
    fn rejects_other_json() {
        assert!(import("{\"hello\": 1}", &ImportOptions::default()).is_err());
        assert!(import("nope", &ImportOptions::default()).is_err());
    }
}
