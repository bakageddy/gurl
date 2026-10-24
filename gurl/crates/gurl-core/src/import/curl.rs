//! cURL command lines → HURL entries. Understands what browsers' "Copy as cURL" and docs produce:
//! shell quoting (`'…'`, `"…"`, `$'…'`, backslashes, `\`-newline continuations), several commands
//! in one paste, and the options that have a HURL equivalent. Unknown options become warnings.

use super::{Folder, ImportError, ImportOptions, ImportResult, Request, layout};
use crate::model::{BodyM, EntryM, Kv, MultipartM, RequestM, ResponseM};

pub fn import(text: &str, options: &ImportOptions) -> Result<ImportResult, ImportError> {
    let commands = split_commands(text);
    let mut root = Folder::new("curl");
    let mut out = ImportResult::default();
    for (i, argv) in commands.iter().enumerate() {
        match parse_command(argv) {
            Ok((entry, name, warnings)) => {
                for w in warnings {
                    out.warnings.push(if commands.len() > 1 {
                        format!("command {}: {w}", i + 1)
                    } else {
                        w
                    });
                }
                root.requests.push(Request { name, entry });
            }
            Err(e) => {
                if commands.len() == 1 {
                    return Err(e);
                }
                out.warnings.push(format!("command {} skipped: {e}", i + 1));
            }
        }
    }
    if root.requests.is_empty() {
        return Err(ImportError::Invalid("no curl command found".to_string()));
    }
    layout(
        &root,
        &ImportOptions {
            one_file: options.one_file || root.requests.len() == 1,
        },
        &mut out,
    );
    Ok(out)
}

/// Converts one command and returns `(entry, name, warnings)`. Public for the Quick Request bar.
pub fn parse_command(argv: &[String]) -> Result<(EntryM, String, Vec<String>), ImportError> {
    let mut args = argv.iter().map(String::as_str).peekable();
    match args.peek() {
        Some(first)
            if first.rsplit(['/', '\\']).next() == Some("curl") || first.ends_with("curl.exe") =>
        {
            args.next();
        }
        _ => return Err(ImportError::Invalid("not a curl command".to_string())),
    }

    let mut req = RequestM {
        method: String::new(),
        ..Default::default()
    };
    let mut url: Option<String> = None;
    let mut data: Vec<String> = Vec::new();
    let mut data_urlencode: Vec<(String, String)> = Vec::new();
    let mut get = false;
    let mut head = false;
    let mut json_flag = false;
    let mut warnings = Vec::new();

    // Options with a value; everything else listed is a flag.
    const WITH_VALUE: &[&str] = &[
        "-X",
        "--request",
        "-H",
        "--header",
        "-d",
        "--data",
        "--data-raw",
        "--data-binary",
        "--data-ascii",
        "--data-urlencode",
        "--json",
        "-F",
        "--form",
        "--form-string",
        "-u",
        "--user",
        "-A",
        "--user-agent",
        "-e",
        "--referer",
        "-b",
        "--cookie",
        "--url",
        "--max-redirs",
        "-m",
        "--max-time",
        "--connect-timeout",
        "-x",
        "--proxy",
        "--retry",
        "--resolve",
        "--cacert",
        "-E",
        "--cert",
        "--key",
        "--limit-rate",
        "--connect-to",
        "--unix-socket",
        "--aws-sigv4",
        "-o",
        "--output",
        "-c",
        "--cookie-jar",
        "-w",
        "--write-out",
        "--max-filesize",
        "--retry-delay",
        "--retry-max-time",
        "--noproxy",
        "--pinnedpubkey",
        "--interface",
        "--dns-servers",
        "-T",
        "--upload-file",
        "-r",
        "--range",
        "--ciphers",
        "--tlsv1.2",
        "--proxy-user",
        "-U",
    ];

    let mut toks: Vec<String> = args.map(str::to_string).collect();
    // `-sS`, `-skL`: split combined short flags (value-taking shorts keep their suffix as value).
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i].clone();
        if t.len() > 2 && t.starts_with('-') && !t.starts_with("--") {
            let (flag, rest) = t.split_at(2);
            if WITH_VALUE.contains(&flag) {
                toks[i] = flag.to_string();
                toks.insert(i + 1, rest.to_string());
            } else {
                let mut expanded: Vec<String> = rest.chars().map(|c| format!("-{c}")).collect();
                toks[i] = flag.to_string();
                for (k, e) in expanded.drain(..).enumerate() {
                    toks.insert(i + 1 + k, e);
                }
            }
        } else if let Some((flag, value)) = t
            .split_once('=')
            .filter(|(f, _)| f.starts_with("--") && WITH_VALUE.contains(f))
        {
            toks[i] = flag.to_string();
            toks.insert(i + 1, value.to_string());
        }
        i += 1;
    }

    let mut it = toks.into_iter();
    while let Some(t) = it.next() {
        let mut value = |name: &str| -> Result<String, ImportError> {
            it.next()
                .ok_or_else(|| ImportError::Invalid(format!("{name} needs a value")))
        };
        match t.as_str() {
            "-X" | "--request" => req.method = value(&t)?.to_uppercase(),
            "--url" => url = Some(value(&t)?),
            "-H" | "--header" => {
                let h = value(&t)?;
                match h.split_once(':') {
                    Some((name, v)) => {
                        let name = name.trim();
                        let v = v.trim();
                        if name.eq_ignore_ascii_case("cookie") {
                            req.cookies.extend(cookie_pairs(v));
                        } else if v.is_empty() && h.trim_end().ends_with(';') {
                            // `-H "X-Empty;"` sends an empty header.
                            req.headers.push(kv(name.trim_end_matches(';'), ""));
                        } else {
                            req.headers.push(kv(name, v));
                        }
                    }
                    None => warnings.push(format!("ignored header without ':': {h}")),
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                data.push(value(&t)?)
            }
            "--data-urlencode" => {
                let v = value(&t)?;
                match v.split_once('=') {
                    Some((n, val)) if !n.is_empty() => {
                        data_urlencode.push((n.to_string(), val.to_string()))
                    }
                    _ => data.push(v),
                }
            }
            "--json" => {
                json_flag = true;
                data.push(value(&t)?);
            }
            "-F" | "--form" | "--form-string" => {
                let f = value(&t)?;
                let Some((name, val)) = f.split_once('=') else {
                    warnings.push(format!("ignored form part without '=': {f}"));
                    continue;
                };
                if t != "--form-string" && (val.starts_with('@') || val.starts_with('<')) {
                    let spec = &val[1..];
                    let (file, ct) = match spec.split_once(";type=") {
                        Some((f, ct)) => (f, Some(ct.to_string())),
                        None => (spec.split(';').next().unwrap_or(spec), None),
                    };
                    req.multipart.push(MultipartM {
                        name: name.to_string(),
                        kind: "file".to_string(),
                        value: file.to_string(),
                        content_type: ct,
                    });
                } else {
                    req.multipart.push(MultipartM {
                        name: name.to_string(),
                        kind: "param".to_string(),
                        value: val.to_string(),
                        content_type: None,
                    });
                }
            }
            "-u" | "--user" => {
                let u = value(&t)?;
                let (user, pass) = u.split_once(':').unwrap_or((&u, ""));
                req.basic_auth = Some(kv(user, pass));
            }
            "-A" | "--user-agent" => req.headers.push(kv("User-Agent", &value(&t)?)),
            "-e" | "--referer" => req.headers.push(kv("Referer", &value(&t)?)),
            "-b" | "--cookie" => {
                let c = value(&t)?;
                if c.contains('=') {
                    req.cookies.extend(cookie_pairs(&c));
                } else {
                    warnings.push(format!("cookie file ignored: {c}"));
                }
            }
            "-G" | "--get" => get = true,
            "-I" | "--head" => head = true,
            "-k" | "--insecure" => req.options.push(kv("insecure", "true")),
            "-L" | "--location" => req.options.push(kv("location", "true")),
            "--location-trusted" => req.options.push(kv("location-trusted", "true")),
            "--max-redirs" => req.options.push(kv("max-redirs", &value(&t)?)),
            "-m" | "--max-time" => req.options.push(kv("max-time", &seconds(&value(&t)?))),
            "--connect-timeout" => req
                .options
                .push(kv("connect-timeout", &seconds(&value(&t)?))),
            "-x" | "--proxy" => req.options.push(kv("proxy", &value(&t)?)),
            "--retry" => req.options.push(kv("retry", &value(&t)?)),
            "--retry-delay" => req
                .options
                .push(kv("retry-interval", &seconds(&value(&t)?))),
            "--resolve" => req.options.push(kv("resolve", &value(&t)?)),
            "--connect-to" => req.options.push(kv("connect-to", &value(&t)?)),
            "--cacert" => req.options.push(kv("cacert", &value(&t)?)),
            "-E" | "--cert" => req.options.push(kv("cert", &value(&t)?)),
            "--key" => req.options.push(kv("key", &value(&t)?)),
            "--limit-rate" => req.options.push(kv("limit-rate", &value(&t)?)),
            "--unix-socket" => req.options.push(kv("unix-socket", &value(&t)?)),
            "--aws-sigv4" => req.options.push(kv("aws-sigv4", &value(&t)?)),
            "--pinnedpubkey" => req.options.push(kv("pinnedpubkey", &value(&t)?)),
            "--compressed" => req.options.push(kv("compressed", "true")),
            "--digest" => req.options.push(kv("digest", "true")),
            "--ntlm" => req.options.push(kv("ntlm", "true")),
            "--negotiate" => req.options.push(kv("negotiate", "true")),
            "--path-as-is" => req.options.push(kv("path-as-is", "true")),
            "-4" | "--ipv4" => req.options.push(kv("ipv4", "true")),
            "-6" | "--ipv6" => req.options.push(kv("ipv6", "true")),
            "-0" | "--http1.0" => req.options.push(kv("http1.0", "true")),
            "--http1.1" => req.options.push(kv("http1.1", "true")),
            "--http2" | "--http2-prior-knowledge" => req.options.push(kv("http2", "true")),
            "--http3" | "--http3-only" => req.options.push(kv("http3", "true")),
            "-n" | "--netrc" => req.options.push(kv("netrc", "true")),
            // Flags that don't change the request itself.
            "-s"
            | "--silent"
            | "-S"
            | "--show-error"
            | "-v"
            | "--verbose"
            | "-i"
            | "--include"
            | "-f"
            | "--fail"
            | "--fail-with-body"
            | "-g"
            | "--globoff"
            | "-N"
            | "--no-buffer"
            | "-#"
            | "--progress-bar"
            | "--no-progress-meter"
            | "-q"
            | "--disable"
            | "--ssl-no-revoke"
            | "--tlsv1"
            | "--tlsv1.1"
            | "--tlsv1.3"
            | "-1"
            | "-2"
            | "-3"
            | "--tls-max"
            | "--no-keepalive"
            | "--raw"
            | "-Z"
            | "--parallel"
            | "--styled-output"
            | "--no-styled-output" => {}
            "-o" | "--output" | "-c" | "--cookie-jar" | "-w" | "--write-out" | "--max-filesize"
            | "--retry-max-time" | "--noproxy" | "--interface" | "--dns-servers" | "-T"
            | "--upload-file" | "-r" | "--range" | "--ciphers" | "--tlsv1.2" | "--proxy-user"
            | "-U" => {
                let v = value(&t)?;
                warnings.push(format!(
                    "option {t} {v} has no HURL equivalent and was dropped"
                ));
            }
            other if other.starts_with('-') && other.len() > 1 => {
                warnings.push(format!("unknown option {other} was dropped"));
            }
            other => {
                if url.is_none() {
                    url = Some(other.to_string());
                } else {
                    warnings.push(format!("extra argument ignored: {other}"));
                }
            }
        }
    }

    let url = url.ok_or_else(|| ImportError::Invalid("no URL in the curl command".to_string()))?;
    req.url = url.trim_matches(|c| c == '\'' || c == '"').to_string();
    // Credentials in the URL (`https://user:pw@host`) → [BasicAuth].
    if let Some((scheme, rest)) = req.url.split_once("://")
        && let Some((cred, host)) = rest.split_once('@')
        && !cred.contains('/')
        && req.basic_auth.is_none()
    {
        let (user, pass) = cred.split_once(':').unwrap_or((cred, ""));
        req.basic_auth = Some(kv(user, pass));
        req.url = format!("{scheme}://{host}");
    }

    let has_data = !data.is_empty() || !data_urlencode.is_empty();
    if get && has_data {
        // `-G -d a=1 -d b=2`: the data goes to the query string.
        for d in data
            .drain(..)
            .flat_map(|d| d.split('&').map(str::to_string).collect::<Vec<_>>())
        {
            let (n, v) = d.split_once('=').unwrap_or((&d, ""));
            req.query.push(kv(&percent_decode(n), &percent_decode(v)));
        }
        for (n, v) in data_urlencode.drain(..) {
            req.query.push(kv(&n, &v));
        }
    }
    if req.method.is_empty() {
        req.method = if head {
            "HEAD".into()
        } else if has_data && !get || !req.multipart.is_empty() {
            "POST".into()
        } else {
            "GET".into()
        };
    }

    let content_type = req
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("content-type"))
        .map(|h| h.value.to_ascii_lowercase());
    if json_flag {
        if content_type.is_none() {
            req.headers.push(kv("Content-Type", "application/json"));
        }
        if !req
            .headers
            .iter()
            .any(|h| h.name.eq_ignore_ascii_case("accept"))
        {
            req.headers.push(kv("Accept", "application/json"));
        }
    }
    if !data.is_empty() || !data_urlencode.is_empty() {
        let joined = data.join("&");
        let is_json = json_flag
            || content_type.as_deref().is_some_and(|c| c.contains("json"))
            || (content_type.is_none()
                && data_urlencode.is_empty()
                && serde_json::from_str::<serde_json::Value>(&joined).is_ok()
                && joined.trim_start().starts_with(['{', '[']));
        if let Some(file) = joined
            .strip_prefix('@')
            .filter(|_| data.len() == 1 && data_urlencode.is_empty())
        {
            req.body = Some(BodyM {
                kind: "file".into(),
                text: file.to_string(),
            });
            warnings.push(format!(
                "body file {file} must live in the scenario's folder"
            ));
        } else if is_json {
            // HURL's JSON body must be valid JSON; otherwise keep it as text.
            let kind = if serde_json::from_str::<serde_json::Value>(&joined).is_ok() {
                "json"
            } else {
                "text"
            };
            req.body = Some(BodyM {
                kind: kind.into(),
                text: joined,
            });
            // Content-Type is implied by a JSON body in HURL.
            if kind == "json" {
                req.headers
                    .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
            }
        } else if content_type
            .as_deref()
            .is_none_or(|c| c.contains("x-www-form-urlencoded"))
            && looks_like_form(&joined)
        {
            for pair in joined.split('&').filter(|p| !p.is_empty()) {
                let (n, v) = pair.split_once('=').unwrap_or((pair, ""));
                req.form.push(kv(&percent_decode(n), &percent_decode(v)));
            }
            for (n, v) in data_urlencode {
                req.form.push(kv(&n, &v));
            }
            // HURL's [Form] sets the content type itself.
            req.headers
                .retain(|h| !h.name.eq_ignore_ascii_case("content-type"));
        } else {
            req.body = Some(BodyM {
                kind: "text".into(),
                text: joined,
            });
        }
    }

    let name = url_name(&req.method, &req.url);
    let entry = EntryM {
        comments: vec![],
        request: req,
        response: Some(ResponseM {
            version: "HTTP".into(),
            status: "*".into(),
            ..Default::default()
        }),
    };
    Ok((entry, name, warnings))
}

fn kv(name: &str, value: &str) -> Kv {
    Kv {
        name: name.to_string(),
        value: value.to_string(),
    }
}

fn cookie_pairs(s: &str) -> Vec<Kv> {
    s.split(';')
        .filter_map(|p| {
            let (n, v) = p.trim().split_once('=')?;
            Some(kv(n.trim(), v.trim()))
        })
        .collect()
}

/// curl takes seconds (fractions allowed); HURL options take a duration with a unit.
fn seconds(v: &str) -> String {
    match v.parse::<f64>() {
        Ok(f) if f.fract() == 0.0 => format!("{}s", f as u64),
        Ok(f) => format!("{}ms", (f * 1000.0).round() as u64),
        Err(_) => v.to_string(),
    }
}

fn looks_like_form(s: &str) -> bool {
    !s.contains('\n') && s.split('&').all(|p| p.contains('=') && !p.starts_with('='))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                Ok(b) => {
                    out.push(b);
                    i += 2;
                }
                Err(_) => out.push(b'%'),
            },
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// `GET users` style name from the URL's last meaningful path segment.
fn url_name(method: &str, url: &str) -> String {
    let path = url.split("://").nth(1).unwrap_or(url);
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let seg = path
        .split('/')
        .skip(1)
        .filter(|s| !s.is_empty() && !s.starts_with("{{"))
        .last()
        .unwrap_or("request");
    format!("{} {}", method.to_lowercase(), seg)
}

// -------------------------------------------------------------- tokenizer

/// Splits pasted text into commands (one argv each), honouring shell quoting, `\`-newline and
/// `^`-newline continuations, and `;` / `&&` / newlines between commands.
pub fn split_commands(text: &str) -> Vec<Vec<String>> {
    let text = text.replace("\r\n", "\n");
    let mut commands: Vec<Vec<String>> = Vec::new();
    let mut argv: Vec<String> = Vec::new();
    let mut tok = String::new();
    let mut in_tok = false;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    let flush_tok = |tok: &mut String, in_tok: &mut bool, argv: &mut Vec<String>| {
        if *in_tok {
            argv.push(std::mem::take(tok));
            *in_tok = false;
        }
    };
    let flush_cmd = |argv: &mut Vec<String>, commands: &mut Vec<Vec<String>>| {
        if !argv.is_empty() {
            commands.push(std::mem::take(argv));
        }
    };

    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if i + 1 < chars.len() && chars[i + 1] == '\n' => i += 2,
            '^' if i + 1 < chars.len() && chars[i + 1] == '\n' => i += 2,
            '\'' => {
                in_tok = true;
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    tok.push(chars[i]);
                    i += 1;
                }
                i += 1;
            }
            '"' => {
                in_tok = true;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\'
                        && i + 1 < chars.len()
                        && matches!(chars[i + 1], '"' | '\\' | '$' | '`' | '\n')
                    {
                        if chars[i + 1] != '\n' {
                            tok.push(chars[i + 1]);
                        }
                        i += 2;
                    } else {
                        tok.push(chars[i]);
                        i += 1;
                    }
                }
                i += 1;
            }
            '$' if i + 1 < chars.len() && chars[i + 1] == '\'' => {
                // ANSI-C quoting.
                in_tok = true;
                i += 2;
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        let e = chars[i + 1];
                        i += 2;
                        match e {
                            'n' => tok.push('\n'),
                            't' => tok.push('\t'),
                            'r' => tok.push('\r'),
                            '\\' => tok.push('\\'),
                            '\'' => tok.push('\''),
                            '"' => tok.push('"'),
                            'x' => {
                                let hex: String = chars[i..].iter().take(2).collect();
                                if let Ok(b) = u8::from_str_radix(&hex, 16) {
                                    tok.push(b as char);
                                    i += hex.len();
                                }
                            }
                            other => {
                                tok.push('\\');
                                tok.push(other);
                            }
                        }
                    } else {
                        tok.push(chars[i]);
                        i += 1;
                    }
                }
                i += 1;
            }
            '\\' if i + 1 < chars.len() => {
                in_tok = true;
                tok.push(chars[i + 1]);
                i += 2;
            }
            ' ' | '\t' => {
                flush_tok(&mut tok, &mut in_tok, &mut argv);
                i += 1;
            }
            '\n' | ';' => {
                flush_tok(&mut tok, &mut in_tok, &mut argv);
                flush_cmd(&mut argv, &mut commands);
                i += 1;
            }
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                flush_tok(&mut tok, &mut in_tok, &mut argv);
                flush_cmd(&mut argv, &mut commands);
                i += 2;
            }
            '#' if !in_tok => {
                // Comment to end of line.
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            _ => {
                in_tok = true;
                tok.push(c);
                i += 1;
            }
        }
    }
    flush_tok(&mut tok, &mut in_tok, &mut argv);
    flush_cmd(&mut argv, &mut commands);
    commands
        .into_iter()
        .filter(|c| {
            c.first().is_some_and(|f| {
                f.rsplit(['/', '\\'])
                    .next()
                    .is_some_and(|b| b == "curl" || b == "curl.exe")
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::emit;

    fn hurl(cmd: &str) -> String {
        let out = import(cmd, &ImportOptions::default()).unwrap();
        out.files[0].text.clone()
    }

    #[test]
    fn tokenizes_quotes_and_continuations() {
        let cmds = split_commands(
            "curl 'https://a' \\\n  -H \"X: y \\\"z\\\"\" $'a\\nb' && curl https://b ; echo no",
        );
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], ["curl", "https://a", "-H", "X: y \"z\"", "a\nb"]);
        assert_eq!(cmds[1], ["curl", "https://b"]);
    }

    #[test]
    fn get_with_headers_and_cookies() {
        let text = hurl(
            "curl -sS 'https://api.example.com/users?x=1' -H 'Accept: application/json' -H 'Cookie: a=1; b=2' -k -L --compressed",
        );
        assert_eq!(
            text,
            "GET https://api.example.com/users?x=1\nAccept: application/json\n[Cookies]\na: 1\nb: 2\n[Options]\ninsecure: true\nlocation: true\ncompressed: true\nHTTP *\n"
        );
    }

    #[test]
    fn post_json_form_and_multipart() {
        let json =
            hurl(r#"curl -X POST https://h/x -H "Content-Type: application/json" -d '{"a":1}'"#);
        assert_eq!(json, "POST https://h/x\n{\"a\":1}\nHTTP *\n");

        let form =
            hurl("curl https://h/login -d user=bob -d 'pass=s%20t' --data-urlencode 'q=a b'");
        assert_eq!(
            form,
            "POST https://h/login\n[Form]\nuser: bob\npass: s t\nq: a b\nHTTP *\n"
        );

        let multi =
            hurl("curl https://h/up -F 'file=@photo.png;type=image/png' -F name=x -u me:pw");
        assert!(
            multi.contains("[Multipart]\nfile: file,photo.png; image/png\nname: x\n"),
            "{multi}"
        );
        assert!(multi.contains("[BasicAuth]\nme: pw\n"), "{multi}");

        let get = hurl("curl -G https://h/s -d q=hello -d page=2");
        assert_eq!(get, "GET https://h/s\n[Query]\nq: hello\npage: 2\nHTTP *\n");
    }

    #[test]
    fn json_flag_and_timeouts() {
        let text = hurl(
            "curl --json '{\"a\":true}' https://h/j -m 2.5 --connect-timeout 10 --max-redirs 3 -I",
        );
        assert!(
            text.starts_with("HEAD https://h/j\nAccept: application/json\n"),
            "{text}"
        );
        assert!(
            text.contains("[Options]\nmax-time: 2500ms\nconnect-timeout: 10s\nmax-redirs: 3\n"),
            "{text}"
        );
        assert!(text.contains("{\"a\":true}\n"), "{text}");
    }

    #[test]
    fn multiple_commands_and_warnings() {
        let out = import(
            "curl https://a/one -o out.txt\ncurl https://b/two --weird",
            &ImportOptions::default(),
        )
        .unwrap();
        assert_eq!(out.files.len(), 2);
        assert_eq!(out.files[0].rel_path, "get-one.hurl");
        assert_eq!(out.warnings.len(), 2);
        assert!(import("echo hi", &ImportOptions::default()).is_err());
        // Round-trip: the emitted text is what the model says.
        assert_eq!(emit(&out.files[1].doc), out.files[1].text);
    }
}
