//! OpenCollection (opencollection.com, schema 1.0.0) → HURL. Accepts a single bundled YAML or the
//! on-disk layout Bruno writes (`opencollection.yml`, `folder.yml`, one YAML per request,
//! `environments/*.yml`). Assertions are mapped best-effort; scripts are reported.

use std::path::Path;

use serde_yaml::Value;

use super::{
    Folder, ImportError, ImportOptions, ImportResult, Request, layout, path_params_to_templates,
};
use crate::model::{AssertM, BodyM, EntryM, Kv, MultipartM, QueryM, RequestM, ResponseM};
use crate::workspace::EnvVar;

pub fn import_text(text: &str, options: &ImportOptions) -> Result<ImportResult, ImportError> {
    let doc: Value =
        serde_yaml::from_str(text).map_err(|e| ImportError::Invalid(format!("not YAML: {e}")))?;
    let mut ctx = Ctx::default();
    let mut out = ImportResult::default();
    if doc.get("items").is_none()
        && doc.get("opencollection").is_none()
        && doc.get("http").is_none()
    {
        return Err(ImportError::Invalid(
            "not an OpenCollection document (no `opencollection`/`items`)".to_string(),
        ));
    }
    let mut root = Folder::new(
        str_of(doc.get("info").and_then(|i| i.get("name")))
            .as_deref()
            .unwrap_or("opencollection"),
    );
    // Collection-level defaults.
    let defaults = Defaults::from(doc.get("request"));
    if doc.get("http").is_some() {
        // A single request file.
        if let Some(r) = convert_item(&doc, &defaults, &mut ctx) {
            root.requests.push(r);
        }
    } else {
        convert_items(doc.get("items"), &defaults, &mut root, &mut ctx);
    }
    root.vars.extend(defaults.vars.clone());
    out.warnings.extend(ctx.finish());
    layout(&root, options, &mut out);
    Ok(out)
}

/// Bruno's layout: directories are folders, `*.yml` files are requests (or `folder.yml` config).
pub fn import_dir(dir: &Path, options: &ImportOptions) -> Result<ImportResult, ImportError> {
    let mut ctx = Ctx::default();
    let mut out = ImportResult::default();
    let col = read_yaml(&dir.join("opencollection.yml"))
        .or_else(|_| read_yaml(&dir.join("opencollection.yaml")));
    let name = col
        .as_ref()
        .ok()
        .and_then(|c| str_of(c.get("info").and_then(|i| i.get("name"))))
        .or_else(|| dir.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "opencollection".to_string());
    let defaults = col
        .as_ref()
        .ok()
        .map(|c| Defaults::from(c.get("request")))
        .unwrap_or_default();
    let mut root = Folder::new(&name);
    walk_dir(dir, &defaults, &mut root, &mut ctx)?;
    root.vars.extend(defaults.vars.clone());
    // Environments: the first becomes gurl.env, the others gurl.env.<name>.
    let env_dir = dir.join("environments");
    if env_dir.is_dir() {
        let mut files: Vec<_> = std::fs::read_dir(&env_dir)
            .map_err(|e| ImportError::Read {
                path: env_dir.clone(),
                source: e,
            })?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "yml" || x == "yaml"))
            .collect();
        files.sort();
        for (i, f) in files.iter().enumerate() {
            let Ok(env) = read_yaml(f) else { continue };
            let env_name = str_of(env.get("name"))
                .or_else(|| f.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let vars = env_variables(env.get("variables"), &mut ctx);
            if vars.is_empty() {
                continue;
            }
            if i == 0 {
                root.vars.extend(vars);
                ctx.warnings
                    .push(format!("environment \"{env_name}\" became gurl.env"));
            } else {
                out.envs.push(super::ImportedEnv {
                    rel_dir: String::new(),
                    file_name: format!("gurl.env.{}", super::slug(&env_name)),
                    vars,
                });
                ctx.warnings.push(format!("environment \"{env_name}\" saved as gurl.env.{} (rename it to gurl.env to use it)", super::slug(&env_name)));
            }
        }
    }
    if root.requests.is_empty() && root.folders.is_empty() {
        return Err(ImportError::Invalid(format!(
            "no request found below {}",
            dir.display()
        )));
    }
    out.warnings.extend(ctx.finish());
    layout(&root, options, &mut out);
    Ok(out)
}

fn walk_dir(
    dir: &Path,
    defaults: &Defaults,
    folder: &mut Folder,
    ctx: &mut Ctx,
) -> Result<(), ImportError> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| ImportError::Read {
            path: dir.to_path_buf(),
            source: e,
        })?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    let mut defaults = defaults.clone();
    if let Ok(cfg) = read_yaml(&dir.join("folder.yml")) {
        if let Some(n) = str_of(cfg.get("info").and_then(|i| i.get("name"))) {
            folder.name = n;
        }
        defaults = defaults.merged(Defaults::from(cfg.get("request")));
        folder.vars.extend(Defaults::from(cfg.get("request")).vars);
    }
    // Requests sorted by `info.seq` when present.
    let mut requests: Vec<(f64, Request)> = Vec::new();
    for p in entries {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if p.is_dir() {
            if name == "environments" || name.starts_with('.') || name == "node_modules" {
                continue;
            }
            let mut sub = Folder::new(&name);
            walk_dir(&p, &defaults, &mut sub, ctx)?;
            folder.folders.push(sub);
        } else if (name.ends_with(".yml") || name.ends_with(".yaml"))
            && !matches!(
                name.as_str(),
                "folder.yml" | "folder.yaml" | "opencollection.yml" | "opencollection.yaml"
            )
        {
            let Ok(doc) = read_yaml(&p) else {
                ctx.warnings
                    .push(format!("{}: not valid YAML, skipped", p.display()));
                continue;
            };
            let seq = doc
                .get("info")
                .and_then(|i| i.get("seq"))
                .and_then(Value::as_f64)
                .unwrap_or(f64::MAX);
            if let Some(r) = convert_item(&doc, &defaults, ctx) {
                requests.push((seq, r));
            }
        }
    }
    requests.sort_by(|a, b| a.0.total_cmp(&b.0));
    folder.requests.extend(requests.into_iter().map(|(_, r)| r));
    Ok(())
}

fn read_yaml(path: &Path) -> Result<Value, ImportError> {
    let text = std::fs::read_to_string(path).map_err(|e| ImportError::Read {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_yaml::from_str(&text)
        .map_err(|e| ImportError::Invalid(format!("{}: {e}", path.display())))
}

#[derive(Default)]
struct Ctx {
    warnings: Vec<String>,
    scripts: usize,
    unsupported: Vec<String>,
}

impl Ctx {
    fn finish(mut self) -> Vec<String> {
        if self.scripts > 0 {
            self.warnings.push(format!(
                "{} script(s) have no HURL equivalent and were dropped",
                self.scripts
            ));
        }
        if !self.unsupported.is_empty() {
            self.unsupported.sort();
            self.unsupported.dedup();
            self.warnings.push(format!(
                "skipped item types: {}",
                self.unsupported.join(", ")
            ));
        }
        self.warnings
    }
}

/// Collection / folder `request` defaults: headers and auth inherited by requests, variables.
#[derive(Clone, Default)]
struct Defaults {
    headers: Vec<Kv>,
    auth: Option<Value>,
    vars: Vec<EnvVar>,
}

impl Defaults {
    fn from(v: Option<&Value>) -> Self {
        let Some(v) = v else {
            return Defaults::default();
        };
        Defaults {
            headers: kv_list(v.get("headers")),
            auth: v.get("auth").cloned(),
            vars: variables(v.get("variables")),
        }
    }
    fn merged(mut self, child: Defaults) -> Self {
        self.headers.extend(child.headers);
        if child.auth.is_some() {
            self.auth = child.auth;
        }
        self.vars.extend(child.vars);
        self
    }
}

fn str_of(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn kv_list(v: Option<&Value>) -> Vec<Kv> {
    v.and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter(|h| !h.get("disabled").and_then(Value::as_bool).unwrap_or(false))
                .filter_map(|h| {
                    Some(Kv {
                        name: str_of(h.get("name"))?,
                        value: str_of(h.get("value")).unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn variables(v: Option<&Value>) -> Vec<EnvVar> {
    v.and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter(|h| !h.get("disabled").and_then(Value::as_bool).unwrap_or(false))
                .filter_map(|h| {
                    Some(EnvVar {
                        name: str_of(h.get("name"))?,
                        value: str_of(h.get("value")).unwrap_or_default(),
                        secret: h.get("secret").and_then(Value::as_bool).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn env_variables(v: Option<&Value>, ctx: &mut Ctx) -> Vec<EnvVar> {
    let vars = variables(v);
    if vars.iter().any(|v| v.secret) {
        ctx.warnings.push(
            "secret environment variables were imported with empty values; fill them in gurl.env"
                .to_string(),
        );
    }
    vars
}

fn convert_items(items: Option<&Value>, defaults: &Defaults, folder: &mut Folder, ctx: &mut Ctx) {
    let Some(items) = items.and_then(Value::as_sequence) else {
        return;
    };
    for item in items {
        let kind = str_of(item.get("info").and_then(|i| i.get("type")))
            .or_else(|| str_of(item.get("type")))
            .unwrap_or_default();
        if kind == "folder" {
            let mut sub = Folder::new(
                str_of(item.get("info").and_then(|i| i.get("name")))
                    .as_deref()
                    .unwrap_or("folder"),
            );
            let child_defaults = defaults.clone().merged(Defaults::from(item.get("request")));
            sub.vars = Defaults::from(item.get("request")).vars;
            convert_items(item.get("items"), &child_defaults, &mut sub, ctx);
            folder.folders.push(sub);
        } else if let Some(r) = convert_item(item, defaults, ctx) {
            folder.requests.push(r);
        }
    }
}

/// One `http` / `graphql` item → a request; other kinds are counted and skipped.
fn convert_item(item: &Value, defaults: &Defaults, ctx: &mut Ctx) -> Option<Request> {
    let info = item.get("info");
    let kind = str_of(info.and_then(|i| i.get("type")))
        .or_else(|| str_of(item.get("type")))
        .unwrap_or_else(|| "http".into());
    let name = str_of(info.and_then(|i| i.get("name"))).unwrap_or_else(|| "request".into());
    let (http, graphql) = match kind.as_str() {
        "http" => (item.get("http")?, false),
        "graphql" => (item.get("graphql")?, true),
        other => {
            ctx.unsupported.push(other.to_string());
            return None;
        }
    };
    let mut req = RequestM {
        method: str_of(http.get("method"))
            .map(|m| m.to_uppercase())
            .unwrap_or_else(|| if graphql { "POST".into() } else { "GET".into() }),
        url: path_params_to_templates(&str_of(http.get("url")).unwrap_or_default()),
        ..Default::default()
    };
    let mut comments = vec![name.clone()];
    if let Some(d) =
        str_of(info.and_then(|i| i.get("description"))).or_else(|| str_of(item.get("docs")))
    {
        comments.extend(d.trim().lines().map(str::to_string));
    }
    req.headers.extend(defaults.headers.iter().cloned());
    req.headers.extend(kv_list(http.get("headers")));
    if let Some(params) = http.get("params").and_then(Value::as_sequence) {
        for p in params {
            if p.get("disabled").and_then(Value::as_bool).unwrap_or(false) {
                continue;
            }
            let Some(n) = str_of(p.get("name")) else {
                continue;
            };
            let v = str_of(p.get("value")).unwrap_or_default();
            if str_of(p.get("type")).as_deref() == Some("path") {
                // `:id` already became `{{id}}`; a value here is a default for the variable.
                if !v.is_empty() {
                    req.options.push(Kv {
                        name: "variable".into(),
                        value: format!("{n}={v}"),
                    });
                }
            } else {
                req.query.push(Kv { name: n, value: v });
            }
        }
    }
    if let Some(body) = http.get("body") {
        convert_body(body, graphql, &mut req, ctx);
    }
    let auth = match http.get("auth") {
        Some(Value::String(s)) if s == "inherit" => defaults.auth.as_ref(),
        Some(a) => Some(a),
        None => defaults.auth.as_ref(),
    };
    if let Some(a) = auth {
        apply_auth(a, &mut req, ctx);
    }
    if let Some(settings) = item.get("settings") {
        if let Some(t) = settings
            .get("timeout")
            .and_then(Value::as_f64)
            .filter(|t| *t > 0.0)
        {
            req.options.push(Kv {
                name: "max-time".into(),
                value: format!("{}ms", t as u64),
            });
        }
        if settings.get("followRedirects").and_then(Value::as_bool) == Some(true) {
            req.options.push(Kv {
                name: "location".into(),
                value: "true".into(),
            });
        }
        if let Some(m) = settings.get("maxRedirects").and_then(Value::as_u64) {
            req.options.push(Kv {
                name: "max-redirs".into(),
                value: m.to_string(),
            });
        }
    }
    let mut response = ResponseM {
        version: "HTTP".into(),
        status: "*".into(),
        ..Default::default()
    };
    if let Some(runtime) = item.get("runtime") {
        ctx.scripts += runtime
            .get("scripts")
            .and_then(Value::as_sequence)
            .map(Vec::len)
            .unwrap_or(0);
        for v in variables(runtime.get("variables")) {
            req.options.push(Kv {
                name: "variable".into(),
                value: format!("{}={}", v.name, v.value),
            });
        }
        if let Some(asserts) = runtime.get("assertions").and_then(Value::as_sequence) {
            for a in asserts {
                if a.get("disabled").and_then(Value::as_bool).unwrap_or(false) {
                    continue;
                }
                match convert_assertion(a, &mut response) {
                    Ok(()) => {}
                    Err(line) => comments.push(format!("TODO assertion not converted: {line}")),
                }
            }
        }
    }
    Some(Request {
        name,
        entry: EntryM {
            comments,
            request: req,
            response: Some(response),
        },
    })
}

fn convert_body(body: &Value, graphql: bool, req: &mut RequestM, ctx: &mut Ctx) {
    // `body` may be a list of variants with `selected`.
    let body = match body.as_sequence() {
        Some(variants) => match variants
            .iter()
            .find(|v| v.get("selected").and_then(Value::as_bool) == Some(true))
            .or(variants.first())
        {
            Some(v) => v.get("body").unwrap_or(v),
            None => return,
        },
        None => body,
    };
    if graphql || body.get("query").is_some() && body.get("type").is_none() {
        let query = str_of(body.get("query")).unwrap_or_default();
        let mut text = query.trim_end().to_string();
        if let Some(v) = str_of(body.get("variables"))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty() && v != "{}")
        {
            text.push_str("\n\nvariables ");
            text.push_str(&v);
        }
        if !text.trim().is_empty() {
            req.body = Some(BodyM {
                kind: "graphql".into(),
                text,
            });
        }
        return;
    }
    let kind = str_of(body.get("type")).unwrap_or_default();
    match kind.as_str() {
        "json" | "text" | "xml" | "sparql" => {
            let data = str_of(body.get("data")).unwrap_or_default();
            if data.trim().is_empty() {
                return;
            }
            let k = match kind.as_str() {
                "json" if serde_json::from_str::<serde_json::Value>(&data).is_ok() => "json",
                "json" => {
                    ctx.warnings.push("a JSON body isn't valid JSON (variables inside strings?) and was kept as text".to_string());
                    "text"
                }
                "xml" => "xml",
                _ => "text",
            };
            if k == "json" {
                req.headers
                    .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
            }
            req.body = Some(BodyM {
                kind: k.into(),
                text: data,
            });
        }
        "form-urlencoded" => {
            req.form.extend(kv_list(body.get("data")));
            req.headers
                .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
        }
        "multipart-form" => {
            if let Some(parts) = body.get("data").and_then(Value::as_sequence) {
                for p in parts {
                    if p.get("disabled").and_then(Value::as_bool).unwrap_or(false) {
                        continue;
                    }
                    let Some(n) = str_of(p.get("name")) else {
                        continue;
                    };
                    let value = match p.get("value") {
                        Some(Value::Sequence(s)) => {
                            s.first().and_then(|v| v.as_str()).unwrap_or("").to_string()
                        }
                        v => str_of(v).unwrap_or_default(),
                    };
                    if str_of(p.get("type")).as_deref() == Some("file") {
                        let file = value
                            .rsplit(['/', '\\'])
                            .next()
                            .unwrap_or(&value)
                            .to_string();
                        ctx.warnings.push(format!(
                            "multipart file {file} must be copied next to the scenario"
                        ));
                        req.multipart.push(MultipartM {
                            name: n,
                            kind: "file".into(),
                            value: file,
                            content_type: str_of(p.get("contentType")),
                        });
                    } else {
                        req.multipart.push(MultipartM {
                            name: n,
                            kind: "param".into(),
                            value,
                            content_type: None,
                        });
                    }
                }
            }
            req.headers
                .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
        }
        "file" => {
            let first = body.get("data").and_then(Value::as_sequence).and_then(|s| {
                s.iter()
                    .find(|f| f.get("selected").and_then(Value::as_bool) != Some(false))
            });
            if let Some(f) = first
                && let Some(path) = str_of(f.get("filePath"))
            {
                let file = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
                ctx.warnings.push(format!(
                    "body file {file} must be copied next to the scenario"
                ));
                req.body = Some(BodyM {
                    kind: "file".into(),
                    text: file,
                });
            }
        }
        "" => {}
        other => ctx
            .warnings
            .push(format!("body type {other} is not supported")),
    }
}

fn apply_auth(auth: &Value, req: &mut RequestM, ctx: &mut Ctx) {
    let kind = str_of(auth.get("type")).unwrap_or_default();
    let s = |k: &str| str_of(auth.get(k)).unwrap_or_default();
    match kind.as_str() {
        "basic" => {
            req.basic_auth = Some(Kv {
                name: s("username"),
                value: s("password"),
            });
        }
        "bearer" => req.headers.push(Kv {
            name: "Authorization".into(),
            value: format!("Bearer {}", s("token")),
        }),
        "apikey" => {
            let kv = Kv {
                name: s("key"),
                value: s("value"),
            };
            if s("placement") == "query" {
                req.query.push(kv);
            } else {
                req.headers.push(kv);
            }
        }
        "digest" | "ntlm" => {
            req.options.push(Kv {
                name: kind.clone(),
                value: "true".into(),
            });
            req.basic_auth = Some(Kv {
                name: s("username"),
                value: s("password"),
            });
        }
        "awsv4" => {
            req.options.push(Kv {
                name: "aws-sigv4".into(),
                value: format!("aws:amz:{}:{}", s("region"), s("service")),
            });
            req.basic_auth = Some(Kv {
                name: s("accessKeyId"),
                value: s("secretAccessKey"),
            });
        }
        "" | "none" | "inherit" => {}
        other => ctx.warnings.push(format!(
            "auth type {other} is not supported; add the header by hand"
        )),
    }
}

/// `res.status eq 200`, `res.body.user.id isNumber`, `res.headers.content-type contains json`…
fn convert_assertion(a: &Value, response: &mut ResponseM) -> Result<(), String> {
    let expr = str_of(a.get("expression")).unwrap_or_default();
    let op = str_of(a.get("operator")).unwrap_or_default();
    let value = str_of(a.get("value")).unwrap_or_default();
    let line = format!("{expr} {op} {value}").trim().to_string();

    let query = if expr == "res.status" {
        QueryM {
            kind: "status".into(),
            args: String::new(),
        }
    } else if let Some(h) = expr.strip_prefix("res.headers.").or_else(|| {
        expr.strip_prefix("res.headers[\"")
            .and_then(|h| h.strip_suffix("\"]"))
    }) {
        QueryM {
            kind: "header".into(),
            args: format!("\"{h}\""),
        }
    } else if expr == "res.body" {
        QueryM {
            kind: "body".into(),
            args: String::new(),
        }
    } else if let Some(p) = expr
        .strip_prefix("res.body.")
        .or_else(|| expr.strip_prefix("res.body"))
    {
        let path = if p.starts_with('[') {
            format!("${p}")
        } else {
            format!("$.{p}")
        };
        QueryM {
            kind: "jsonpath".into(),
            args: format!("\"{path}\""),
        }
    } else if expr == "res.responseTime" {
        QueryM {
            kind: "duration".into(),
            args: String::new(),
        }
    } else {
        return Err(line);
    };

    let lit = |v: &str| -> String {
        let t = v.trim();
        if t.is_empty() {
            return String::new();
        }
        if t.parse::<f64>().is_ok()
            || t == "true"
            || t == "false"
            || t == "null"
            || t.starts_with("{{")
        {
            return t.to_string();
        }
        let t = t.trim_matches(|c| c == '"' || c == '\'');
        format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
    };
    let (not, func, val) = match op.as_str() {
        "eq" => (false, "==", lit(&value)),
        "neq" => (false, "!=", lit(&value)),
        "gt" => (false, ">", lit(&value)),
        "gte" => (false, ">=", lit(&value)),
        "lt" => (false, "<", lit(&value)),
        "lte" => (false, "<=", lit(&value)),
        "contains" => (false, "contains", lit(&value)),
        "notContains" => (true, "contains", lit(&value)),
        "startsWith" => (false, "startsWith", lit(&value)),
        "endsWith" => (false, "endsWith", lit(&value)),
        "matches" => (
            false,
            "matches",
            format!("/{}/", value.trim().trim_matches('/')),
        ),
        "isDefined" => (false, "exists", String::new()),
        "isUndefined" => (true, "exists", String::new()),
        "isNull" => (false, "==", "null".into()),
        "isString" => (false, "isString", String::new()),
        "isNumber" => (false, "isNumber", String::new()),
        "isBoolean" => (false, "isBoolean", String::new()),
        "isArray" => (false, "isCollection", String::new()),
        "isEmpty" => (false, "isEmpty", String::new()),
        "isNotEmpty" => (true, "isEmpty", String::new()),
        "isTruthy" => (false, "==", "true".into()),
        "isFalsy" => (false, "==", "false".into()),
        _ => return Err(line),
    };
    response.asserts.push(AssertM {
        query,
        filters: vec![],
        not,
        func: func.into(),
        value: val,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
opencollection: "1.0.0"
info:
  name: Sample API
request:
  headers:
    - name: X-Client
      value: gurl
  variables:
    - name: host
      value: https://api.example.com
items:
  - info:
      name: Get Users
      type: http
      seq: 1
    http:
      method: GET
      url: "{{host}}/users/:id"
      params:
        - name: limit
          value: "10"
          type: query
        - name: id
          value: "42"
          type: path
      auth:
        type: bearer
        token: "{{token}}"
    runtime:
      assertions:
        - expression: res.status
          operator: eq
          value: "200"
        - expression: res.body.name
          operator: isString
        - expression: res.headers.content-type
          operator: contains
          value: json
        - expression: res.weird
          operator: eq
          value: 1
      scripts:
        - type: tests
          code: test("x", () => {})
  - info:
      name: Users
      type: folder
    items:
      - info:
          name: Create
          type: http
        http:
          method: post
          url: "{{host}}/users"
          body:
            type: json
            data: |-
              {"name": "x"}
          auth: inherit
        settings:
          timeout: 5000
          followRedirects: true
  - info:
      name: Stream
      type: websocket
"#;

    #[test]
    fn converts_bundled_collection() {
        let out = import_text(SAMPLE, &ImportOptions::default()).unwrap();
        let paths: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(paths, ["get-users.hurl", "users/create.hurl"]);
        let get = &out.files[0].text;
        assert_eq!(
            get,
            "# Get Users\n# TODO assertion not converted: res.weird eq 1\nGET {{host}}/users/{{id}}\nX-Client: gurl\nAuthorization: Bearer {{token}}\n[Query]\nlimit: 10\n[Options]\nvariable: id=42\nHTTP *\n[Asserts]\nstatus == 200\njsonpath \"$.name\" isString\nheader \"content-type\" contains \"json\"\n"
        );
        let create = &out.files[1].text;
        assert!(create.contains("POST {{host}}/users\nX-Client: gurl\n[Options]\nmax-time: 5000ms\nlocation: true\n{\"name\": \"x\"}\n"), "{create}");
        assert_eq!(out.envs[0].vars[0].name, "host");
        assert!(out.warnings.iter().any(|w| w.contains("websocket")));
        assert!(out.warnings.iter().any(|w| w.contains("script")));
    }

    #[test]
    fn reads_bruno_directory_layout() {
        let root = std::env::temp_dir().join(format!("gurl-oc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("auth")).unwrap();
        std::fs::create_dir_all(root.join("environments")).unwrap();
        std::fs::write(
            root.join("opencollection.yml"),
            "opencollection: \"1.0.0\"\ninfo:\n  name: Bruno Shop\n",
        )
        .unwrap();
        std::fs::write(
            root.join("auth").join("folder.yml"),
            "info:\n  name: Auth\n  type: folder\n",
        )
        .unwrap();
        std::fs::write(root.join("auth").join("login.yml"), "info:\n  name: Login\n  type: http\n  seq: 2\nhttp:\n  method: POST\n  url: \"{{host}}/login\"\n  body:\n    type: form-urlencoded\n    data:\n      - name: u\n        value: a\n").unwrap();
        std::fs::write(root.join("auth").join("first.yml"), "info:\n  name: First\n  type: http\n  seq: 1\nhttp:\n  method: GET\n  url: \"{{host}}/first\"\n").unwrap();
        std::fs::write(root.join("environments").join("dev.yml"), "name: dev\nvariables:\n  - name: host\n    value: http://localhost:3000\n  - name: token\n    secret: true\n").unwrap();
        std::fs::write(
            root.join("environments").join("prod.yml"),
            "name: prod\nvariables:\n  - name: host\n    value: https://shop.example.com\n",
        )
        .unwrap();

        let out = import_dir(&root, &ImportOptions::default()).unwrap();
        let paths: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(paths, ["auth/first.hurl", "auth/login.hurl"]);
        assert!(out.files[1].text.contains("[Form]\nu: a\n"));
        let env_files: Vec<&str> = out.envs.iter().map(|e| e.file_name.as_str()).collect();
        assert!(
            env_files.contains(&"gurl.env") && env_files.contains(&"gurl.env.prod"),
            "{env_files:?}"
        );
        let dev = out.envs.iter().find(|e| e.file_name == "gurl.env").unwrap();
        assert!(dev.vars.iter().any(|v| v.name == "token" && v.secret));
        let _ = std::fs::remove_dir_all(&root);
    }
}
