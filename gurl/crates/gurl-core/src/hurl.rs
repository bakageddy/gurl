use crate::error::{RunError, chain_message};
use hurl::http::{CredentialForwarding, FollowLocation, IpResolve, RequestedHttpVersion};
use hurl::runner::{Output, RunnerOptions, RunnerOptionsBuilder, Value, VariableSet};
use hurl::util::logger::{LoggerOptions, LoggerOptionsBuilder, Verbosity};
use hurl::util::path::ContextDir;
use hurl_core::input::Input;
use hurl_core::types::{BytesPerSec, Count};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// All the Hurl runner options exposed to the UI.
/// Most fields map 1:1 to `hurl::runner::RunnerOptions`.
#[derive(Debug, Deserialize, Clone)]
pub struct HurlRunOptions {
    #[serde(default)]
    pub variables: HashMap<String, String>,

    #[serde(default)]
    pub insecure: bool,

    #[serde(default)]
    pub follow_location: bool,
    #[serde(default)]
    pub follow_location_trusted: bool,
    #[serde(default = "default_max_redirect")]
    pub max_redirect: i32,

    #[serde(default)]
    pub compressed: bool,

    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_timeout")]
    pub connect_timeout_ms: u64,

    #[serde(default)]
    pub retry: i32,
    #[serde(default)]
    pub retry_interval_ms: u64,
    #[serde(default)]
    pub delay_ms: u64,

    #[serde(default)]
    pub http_version: String,
    #[serde(default)]
    pub ip_resolve: String,

    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default)]
    pub no_proxy: Option<String>,

    #[serde(default)]
    pub cacert_file: Option<String>,
    #[serde(default)]
    pub client_cert_file: Option<String>,
    #[serde(default)]
    pub client_key_file: Option<String>,

    #[serde(default)]
    pub from_entry: Option<usize>,
    #[serde(default)]
    pub to_entry: Option<usize>,

    #[serde(default = "default_true")]
    pub fail_fast: bool,
    #[serde(default)]
    pub no_assert: bool,
    #[serde(default)]
    pub skip: bool,
    #[serde(default = "default_true")]
    pub use_cookie_store: bool,

    #[serde(default)]
    pub resolves: Vec<String>,
    #[serde(default)]
    pub connects_to: Vec<String>,

    #[serde(default)]
    pub verbosity: String, // "none" | "verbose" | "very-verbose"

    // Authentication & auth methods
    #[serde(default)]
    pub digest: bool,
    #[serde(default)]
    pub negotiate: bool,
    #[serde(default)]
    pub ntlm: bool,
    #[serde(default)]
    pub netrc: bool,
    #[serde(default)]
    pub netrc_optional: bool,
    #[serde(default)]
    pub netrc_file: Option<String>,
    #[serde(default)]
    pub aws_sigv4: Option<String>,

    // TLS / connection knobs
    #[serde(default)]
    pub ssl_no_revoke: bool,
    #[serde(default)]
    pub path_as_is: bool,
    #[serde(default)]
    pub pinned_pub_key: Option<String>,

    // Unix socket / transfer speed / repeat
    #[serde(default)]
    pub unix_socket: Option<String>,
    #[serde(default)]
    pub max_send_speed: Option<u64>,
    #[serde(default)]
    pub max_recv_speed: Option<u64>,
    #[serde(default = "default_repeat")]
    pub repeat: i32,

    #[serde(default = "default_true")]
    pub allow_reuse: bool,
}

fn default_max_redirect() -> i32 {
    50
}
fn default_timeout() -> u64 {
    300_000
}
fn default_true() -> bool {
    true
}
fn default_repeat() -> i32 {
    1
}

impl Default for HurlRunOptions {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            insecure: false,
            follow_location: false,
            follow_location_trusted: false,
            max_redirect: default_max_redirect(),
            compressed: false,
            timeout_ms: default_timeout(),
            connect_timeout_ms: default_timeout(),
            retry: 0,
            retry_interval_ms: 1000,
            delay_ms: 0,
            http_version: String::new(),
            ip_resolve: String::new(),
            user: None,
            user_agent: None,
            proxy: None,
            no_proxy: None,
            cacert_file: None,
            client_cert_file: None,
            client_key_file: None,
            from_entry: None,
            to_entry: None,
            fail_fast: true,
            no_assert: false,
            skip: false,
            use_cookie_store: true,
            resolves: Vec::new(),
            connects_to: Vec::new(),
            verbosity: String::new(),
            digest: false,
            negotiate: false,
            ntlm: false,
            netrc: false,
            netrc_optional: false,
            netrc_file: None,
            aws_sigv4: None,
            ssl_no_revoke: false,
            path_as_is: false,
            pinned_pub_key: None,
            unix_socket: None,
            max_send_speed: None,
            max_recv_speed: None,
            repeat: default_repeat(),
            allow_reuse: true,
        }
    }
}

/// A variable coming from a workspace `gurl.env` chain.
#[derive(Debug, Clone)]
pub struct VarSpec {
    pub name: String,
    pub value: String,
    pub secret: bool,
}

/// Result of running one `.hurl` file or text snippet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRun {
    pub path: String,
    pub ok: bool,
    pub duration_ms: u128,
    pub report: Option<serde_json::Value>,
    pub errors: Vec<String>,
    pub error: Option<String>,
}

/// Everything needed to export a run later (HTML/JUnit/TAP reports).
pub struct Executed {
    pub content: String,
    pub input: Input,
    pub result: hurl::runner::HurlResult,
    pub secrets: Vec<String>,
}

pub enum Source {
    File(PathBuf),
    /// Unsaved text; the optional path gives the label and the directory used to resolve
    /// relative `file,...;` references.
    Text(String, Option<PathBuf>),
}

/// Runs a source with `options` plus the workspace variables `env` (explicit
/// `options.variables` win over `env`).
pub fn run(
    source: Source,
    options: &HurlRunOptions,
    env: &[VarSpec],
) -> (FileRun, Option<Executed>) {
    let start = Instant::now();
    let label = match &source {
        Source::File(p) => p.to_string_lossy().to_string(),
        Source::Text(_, Some(p)) => p.to_string_lossy().to_string(),
        Source::Text(_, None) => "request.hurl".to_string(),
    };
    let fail = |msg: String| {
        (
            FileRun {
                path: label.clone(),
                ok: false,
                duration_ms: start.elapsed().as_millis(),
                report: None,
                errors: vec![],
                error: Some(msg),
            },
            None,
        )
    };

    let (content, input, context_dir) = match source {
        Source::File(path) => {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => return fail(format!("failed to read {}: {e}", path.display())),
            };
            let parent = path.parent().unwrap_or(Path::new(""));
            let context_dir = ContextDir::new(parent, parent);
            (content, Input::new(&path.to_string_lossy()), context_dir)
        }
        Source::Text(text, path) => {
            let dir = path
                .as_ref()
                .and_then(|p| p.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let context_dir = ContextDir::new(&dir, &dir);
            let name = path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "request.hurl".to_string());
            (text, Input::new(&name), context_dir)
        }
    };

    let runner_options = match build_runner_options(options, context_dir) {
        Ok(o) => o,
        Err(e) => return fail(chain_message(&e)),
    };
    let variables = build_variables(env, &options.variables);
    let secrets = variables.secrets();
    let logger_options = build_logger_options(&options.verbosity);

    let result = match hurl::runner::run(
        &content,
        Some(&input),
        &runner_options,
        &variables,
        &logger_options,
    ) {
        Ok(r) => r,
        Err(e) => return fail(format!("hurl run failed: {e}")),
    };

    let errors: Vec<String> = result
        .errors()
        .iter()
        .map(|(err, src)| format!("{}:{}: {:?}", input, src.start.line, err.kind))
        .collect();

    let mut report = match result.to_json(&content, &input, None, &[]) {
        Ok(r) => r,
        Err(e) => return fail(format!("failed to serialize report: {e}")),
    };
    inject_response_bodies(&mut report, &result);
    inject_assert_sources(&mut report, &content);

    let file_run = FileRun {
        path: label,
        ok: result.success && errors.is_empty(),
        duration_ms: start.elapsed().as_millis(),
        report: Some(report),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        },
        errors,
    };
    (
        file_run,
        Some(Executed {
            content,
            input,
            result,
            secrets,
        }),
    )
}

fn count(name: &'static str, n: i32) -> Result<Count, RunError> {
    Count::try_from(n).map_err(|reason| RunError::InvalidOption { name, reason })
}

fn build_runner_options(
    options: &HurlRunOptions,
    context_dir: ContextDir,
) -> Result<RunnerOptions, RunError> {
    let mut builder = RunnerOptionsBuilder::new();

    builder
        .context_dir(&context_dir)
        .allow_reuse(options.allow_reuse)
        .insecure(options.insecure)
        .compressed(options.compressed)
        .timeout(Duration::from_millis(options.timeout_ms))
        .connect_timeout(Duration::from_millis(options.connect_timeout_ms))
        .retry(Some(count("retry", options.retry)?))
        .retry_interval(Duration::from_millis(options.retry_interval_ms))
        .delay(Duration::from_millis(options.delay_ms))
        .max_redirect(count("max_redirect", options.max_redirect)?)
        .fail_fast(options.fail_fast)
        .no_assert(options.no_assert)
        .skip(options.skip)
        .use_cookie_store(options.use_cookie_store)
        .from_entry(options.from_entry)
        .to_entry(options.to_entry)
        .digest(options.digest)
        .negotiate(options.negotiate)
        .ntlm(options.ntlm)
        .netrc(options.netrc)
        .netrc_optional(options.netrc_optional)
        .ssl_no_revoke(options.ssl_no_revoke)
        .path_as_is(options.path_as_is);

    if options.follow_location {
        let forwarding = if options.follow_location_trusted {
            CredentialForwarding::AllHosts
        } else {
            CredentialForwarding::OnlyInitialHost
        };
        builder.follow_location(FollowLocation::Follow(forwarding));
    }

    match options.http_version.as_str() {
        "1.0" => builder.http_version(RequestedHttpVersion::Http10),
        "1.1" => builder.http_version(RequestedHttpVersion::Http11),
        "2" => builder.http_version(RequestedHttpVersion::Http2),
        "3" => builder.http_version(RequestedHttpVersion::Http3),
        _ => builder.http_version(RequestedHttpVersion::default()),
    };

    match options.ip_resolve.as_str() {
        "v4" => builder.ip_resolve(IpResolve::IpV4),
        "v6" => builder.ip_resolve(IpResolve::IpV6),
        _ => builder.ip_resolve(IpResolve::default()),
    };

    if let Some(user) = options.user.as_ref().filter(|s| !s.is_empty()) {
        builder.user(Some(user.clone()));
    }
    if let Some(ua) = options.user_agent.as_ref().filter(|s| !s.is_empty()) {
        builder.user_agent(Some(ua.clone()));
    }
    if let Some(proxy) = options.proxy.as_ref().filter(|s| !s.is_empty()) {
        builder.proxy(Some(proxy.clone()));
    }
    if let Some(no_proxy) = options.no_proxy.as_ref().filter(|s| !s.is_empty()) {
        builder.no_proxy(Some(no_proxy.clone()));
    }
    if let Some(cacert) = options.cacert_file.as_ref().filter(|s| !s.is_empty()) {
        builder.cacert_file(Some(cacert.clone()));
    }
    if let Some(cert) = options.client_cert_file.as_ref().filter(|s| !s.is_empty()) {
        builder.client_cert_file(Some(cert.clone()));
    }
    if let Some(key) = options.client_key_file.as_ref().filter(|s| !s.is_empty()) {
        builder.client_key_file(Some(key.clone()));
    }

    if let Some(netrc_file) = options.netrc_file.as_ref().filter(|s| !s.is_empty()) {
        builder.netrc_file(Some(netrc_file.clone()));
    }
    if let Some(aws) = options.aws_sigv4.as_ref().filter(|s| !s.is_empty()) {
        builder.aws_sigv4(Some(aws.clone()));
    }
    if let Some(pin) = options.pinned_pub_key.as_ref().filter(|s| !s.is_empty()) {
        builder.pinned_pub_key(Some(pin.clone()));
    }
    if let Some(sock) = options.unix_socket.as_ref().filter(|s| !s.is_empty()) {
        builder.unix_socket(Some(sock.clone()));
    }

    builder.repeat(Some(count("repeat", options.repeat)?));

    if let Some(v) = options.max_send_speed.filter(|v| *v > 0) {
        builder.max_send_speed(Some(BytesPerSec(v)));
    }
    if let Some(v) = options.max_recv_speed.filter(|v| *v > 0) {
        builder.max_recv_speed(Some(BytesPerSec(v)));
    }

    builder.resolves(&options.resolves);
    builder.connects_to(&options.connects_to);

    // We don't expose arbitrary output files from the UI; keep responses in memory.
    builder.output(None::<Output>);

    Ok(builder.build())
}

pub(crate) fn parse_value(value: &str) -> Value {
    // Numbers and booleans keep their type, anything else is a string.
    if let Ok(n) = value.parse::<i64>() {
        Value::Number(hurl::runner::Number::Integer(n))
    } else if let Ok(f) = value.parse::<f64>() {
        Value::Number(hurl::runner::Number::Float(f))
    } else if value.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if value.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else {
        Value::String(value.to_string())
    }
}

fn build_variables(env: &[VarSpec], overrides: &HashMap<String, String>) -> VariableSet {
    let mut variables = VariableSet::new();
    for v in env {
        if v.secret {
            variables.insert_secret(v.name.clone(), v.value.clone());
        } else {
            variables.insert(v.name.clone(), parse_value(&v.value));
        }
    }
    for (name, value) in overrides {
        variables.insert(name.clone(), parse_value(value));
    }
    variables
}

fn build_logger_options(verbosity: &str) -> LoggerOptions {
    let v = match verbosity {
        "verbose" => Some(Verbosity::Verbose),
        "very-verbose" => Some(Verbosity::VeryVerbose),
        _ => None,
    };
    LoggerOptionsBuilder::new()
        .color(false)
        .verbosity(v)
        .build()
}

/// Response bodies larger than this are cut in the report sent to the UI.
const BODY_PREVIEW_LIMIT: usize = 2 * 1024 * 1024;

fn inject_response_bodies(json: &mut serde_json::Value, result: &hurl::runner::HurlResult) {
    let Some(entries_json) = json.get_mut("entries").and_then(|e| e.as_array_mut()) else {
        return;
    };
    for (entry_idx, entry_json) in entries_json.iter_mut().enumerate() {
        let Some(calls_json) = entry_json.get_mut("calls").and_then(|c| c.as_array_mut()) else {
            continue;
        };
        for (call_idx, call_json) in calls_json.iter_mut().enumerate() {
            let body_bytes = result
                .entries
                .get(entry_idx)
                .and_then(|e| e.calls.get(call_idx))
                .map(|c| c.response.body.clone())
                .unwrap_or_default();
            let size = body_bytes.len();
            let truncated = size > BODY_PREVIEW_LIMIT;
            let shown = if truncated {
                &body_bytes[..BODY_PREVIEW_LIMIT]
            } else {
                &body_bytes[..]
            };
            let body_text = String::from_utf8_lossy(shown).to_string();
            if let Some(response_json) = call_json.get_mut("response") {
                response_json["body"] = serde_json::Value::String(body_text);
                response_json["bodySize"] = serde_json::Value::from(size);
                response_json["bodyTruncated"] = serde_json::Value::Bool(truncated);
            }
        }
    }
}

/// HURL's report only has the line number of each assert. Add the text of that line so the UI
/// can show the condition itself (`jsonpath "$.id" == 1`, `HTTP 200`, ...).
///
/// `HTTP 200` is two implicit asserts for HURL (version, then status) on one line; they are
/// merged into one row so the UI doesn't show the line twice.
fn inject_assert_sources(json: &mut serde_json::Value, content: &str) {
    let lines: Vec<&str> = content.lines().collect();
    let Some(entries) = json.get_mut("entries").and_then(|e| e.as_array_mut()) else {
        return;
    };
    for entry in entries {
        let Some(asserts) = entry.get_mut("asserts").and_then(|a| a.as_array_mut()) else {
            continue;
        };
        let mut merged: Vec<serde_json::Value> = Vec::new();
        for a in asserts.drain(..) {
            let line = a.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
            if let Some(prev) = merged
                .last_mut()
                .filter(|p| p.get("line").and_then(|l| l.as_u64()) == Some(line))
            {
                let ok = prev["success"].as_bool().unwrap_or(false)
                    && a["success"].as_bool().unwrap_or(false);
                prev["success"] = serde_json::Value::Bool(ok);
                if let Some(msg) = a.get("message").and_then(|m| m.as_str()) {
                    let joined = match prev.get("message").and_then(|m| m.as_str()) {
                        Some(p) if !p.is_empty() => format!("{p}\n{msg}"),
                        _ => msg.to_string(),
                    };
                    prev["message"] = serde_json::Value::String(joined);
                }
                continue;
            }
            merged.push(a);
        }
        for a in merged.iter_mut() {
            let line = a.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
            if let Some(text) = line.checked_sub(1).and_then(|i| lines.get(i)) {
                a["source"] = serde_json::Value::String(text.trim().to_string());
            }
        }
        *asserts = merged;
    }
}

#[cfg(test)]
mod report_tests {
    use super::inject_assert_sources;
    use serde_json::json;

    #[test]
    fn same_line_asserts_are_merged() {
        let mut report = json!({"entries": [{"asserts": [
            {"line": 2, "success": true},
            {"line": 2, "success": false, "message": "status"},
            {"line": 4, "success": true}
        ]}]});
        inject_assert_sources(&mut report, "GET http://x\nHTTP 200\n[Asserts]\nstatus == 200\n");
        let asserts = report["entries"][0]["asserts"].as_array().unwrap();
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0]["source"], "HTTP 200");
        assert_eq!(asserts[0]["success"], false);
        assert_eq!(asserts[0]["message"], "status");
        assert_eq!(asserts[1]["source"], "status == 200");
    }
}
