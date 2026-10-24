//! `gurl cli` end to end, through the real binary: hurl-compatible exit codes, workspace variables
//! and the version report.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;

fn gurl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gurl"))
}

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("gurl-cli-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// A one-shot HTTP server answering `{"n":3}` to `count` requests, on a free port.
fn serve(count: usize) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(count) {
            let mut s = stream.unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let body = r#"{"n":3}"#;
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
    port
}

#[test]
fn version_reports_gurl_hurl_and_libraries() {
    let out = gurl().arg("--version").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("gURL "), "{text}");
    assert!(text.contains("\nhurl 8."), "{text}");
    assert!(text.contains("libcurl/"), "{text}");
    assert!(text.contains("\nlibxml2:  "), "{text}");

    let same = gurl().args(["cli", "--version"]).output().unwrap();
    assert_eq!(String::from_utf8(same.stdout).unwrap(), text);
}

#[test]
fn exit_codes_match_hurl_and_workspace_variables_apply() {
    let port = serve(3);
    let root = tmp("ws");
    let api = root.join("api");
    std::fs::create_dir_all(&api).unwrap();
    std::fs::write(
        root.join("gurl.env"),
        format!("host=127.0.0.1:{port}\nn=0\n"),
    )
    .unwrap();
    std::fs::write(api.join("gurl.env"), "n=3\n").unwrap();
    let ok = api.join("ok.hurl");
    std::fs::write(
        &ok,
        "GET http://{{host}}/\nHTTP 200\n[Asserts]\njsonpath \"$.n\" == {{n}}\n",
    )
    .unwrap();
    let fail = api.join("fail.hurl");
    std::fs::write(
        &fail,
        "GET http://{{host}}/\nHTTP 200\n[Asserts]\njsonpath \"$.n\" == 4\n",
    )
    .unwrap();
    let bad = api.join("bad.hurl");
    std::fs::write(&bad, "GET http://{{host}}/\nHTTP 200\n[Asserts\n").unwrap();

    // 0: the deeper gurl.env wins (n=3), discovered without --workspace.
    let out = gurl().args(["cli", "--test"]).arg(&ok).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // 4: assert failure.
    let out = gurl().args(["cli", "--test"]).arg(&fail).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    // 2: parse error.
    let out = gurl().arg("cli").arg(&bad).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    // 3: runtime error, here an undefined variable because gurl.env is ignored.
    let out = gurl()
        .args(["cli", "--no-workspace"])
        .arg(&ok)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let _ = std::fs::remove_dir_all(&root);
}
