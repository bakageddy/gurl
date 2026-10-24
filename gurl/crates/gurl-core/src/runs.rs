//! Running scopes (text / file / directory), streaming results to the UI, keeping
//! history and exporting HURL's own reports.

use crate::error::RunError;
use crate::hurl::{self, Executed, FileRun, HurlRunOptions, Source, VarSpec};
use crate::workspace;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const MAX_STORED: usize = 20;
const MAX_HISTORY: usize = 30;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeRequest {
    /// `text`, `file` or `dir`.
    pub kind: String,
    pub path: Option<String>,
    /// Unsaved editor text (for `text`, or `file` to run the buffer instead of the disk copy).
    pub content: Option<String>,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub jobs: usize,
    pub options: HurlRunOptions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum RunEvent {
    #[serde(rename_all = "camelCase")]
    Started { run_id: String, files: Vec<String> },
    #[serde(rename_all = "camelCase")]
    FileStarted { index: usize },
    #[serde(rename_all = "camelCase")]
    FileFinished { index: usize, run: FileRun },
    #[serde(rename_all = "camelCase")]
    Done { summary: RunSummary },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub id: String,
    pub label: String,
    pub started_at: u64,
    pub duration_ms: u128,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub files: Vec<FileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSummary {
    pub path: String,
    pub ok: bool,
    pub duration_ms: u128,
    pub entries: usize,
}

/// One file's run result plus what's needed to export it later.
type Outcome = (FileRun, Option<Executed>);

struct StoredRun {
    id: String,
    runs: Vec<FileRun>,
    executed: Vec<Option<Executed>>,
}

#[derive(Default)]
pub struct RunStore {
    stored: Mutex<VecDeque<StoredRun>>,
    cancels: Mutex<Vec<(String, Arc<AtomicBool>)>>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn new_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("run-{nanos:x}")
}

fn env_for(root: Option<&Path>, file: &Path) -> Vec<VarSpec> {
    let (Some(root), Some(dir)) = (root, file.parent()) else {
        return vec![];
    };
    workspace::effective_vars(root, dir)
        .map(|vars| {
            vars.into_iter()
                .map(|v| VarSpec {
                    name: v.name,
                    value: v.value,
                    secret: v.secret,
                })
                .collect()
        })
        .unwrap_or_default()
}

impl RunStore {
    pub fn cancel(&self, run_id: &str) {
        if let Some((_, flag)) = self
            .cancels
            .lock()
            .unwrap()
            .iter()
            .find(|(id, _)| id == run_id)
        {
            flag.store(true, Ordering::SeqCst);
        }
    }

    pub fn execute(
        &self,
        root: Option<PathBuf>,
        req: ScopeRequest,
        on_event: &(dyn Fn(RunEvent) + Sync),
    ) -> Result<RunSummary, RunError> {
        let started = Instant::now();
        let started_at = now_secs();
        let id = new_run_id();
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancels
            .lock()
            .unwrap()
            .push((id.clone(), cancel.clone()));

        // Job list: (display path, source builder inputs)
        let jobs: Vec<(PathBuf, Option<String>)> = match req.kind.as_str() {
            "text" => vec![(
                PathBuf::from(req.path.clone().unwrap_or_else(|| "request.hurl".into())),
                req.content.clone(),
            )],
            "file" => {
                let p = PathBuf::from(req.path.clone().ok_or(RunError::MissingPath)?);
                vec![(p, req.content.clone())]
            }
            "dir" => {
                let dir = PathBuf::from(req.path.clone().ok_or(RunError::MissingPath)?);
                let base = root.clone().unwrap_or_else(|| dir.clone());
                workspace::collect_hurl_files(&base, &dir, req.recursive)?
                    .into_iter()
                    .map(|p| (p, None))
                    .collect()
            }
            other => return Err(RunError::UnknownKind(other.to_string())),
        };
        let label = match req.kind.as_str() {
            "dir" => req.path.clone().unwrap_or_default(),
            _ => jobs
                .first()
                .map(|(p, _)| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        };

        let names: Vec<String> = jobs
            .iter()
            .map(|(p, _)| p.to_string_lossy().to_string())
            .collect();
        on_event(RunEvent::Started {
            run_id: id.clone(),
            files: names,
        });

        let total = jobs.len();
        let slots: Vec<Mutex<Option<Outcome>>> = (0..total).map(|_| Mutex::new(None)).collect();
        let next = AtomicUsize::new(0);
        let workers = if req.kind == "dir" {
            req.jobs.clamp(1, 16).min(total.max(1))
        } else {
            1
        };

        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    loop {
                        if cancel.load(Ordering::SeqCst) {
                            break;
                        }
                        let i = next.fetch_add(1, Ordering::SeqCst);
                        if i >= total {
                            break;
                        }
                        let (path, content) = &jobs[i];
                        on_event(RunEvent::FileStarted { index: i });
                        let source = match (req.kind.as_str(), content) {
                            ("text", Some(c)) => Source::Text(c.clone(), None),
                            (_, Some(c)) => Source::Text(c.clone(), Some(path.clone())),
                            _ => Source::File(path.clone()),
                        };
                        let env = env_for(root.as_deref(), path);
                        let out = hurl::run(source, &req.options, &env);
                        on_event(RunEvent::FileFinished {
                            index: i,
                            run: out.0.clone(),
                        });
                        *slots[i].lock().unwrap() = Some(out);
                    }
                });
            }
        });

        let cancelled = cancel.load(Ordering::SeqCst);
        self.cancels.lock().unwrap().retain(|(rid, _)| rid != &id);

        let mut runs = Vec::new();
        let mut executed = Vec::new();
        for slot in slots {
            if let Some((run, ex)) = slot.into_inner().unwrap() {
                runs.push(run);
                executed.push(ex);
            }
        }
        let files: Vec<FileSummary> = runs
            .iter()
            .zip(&executed)
            .map(|(r, e)| FileSummary {
                path: r.path.clone(),
                ok: r.ok,
                duration_ms: r.duration_ms,
                entries: e.as_ref().map(|e| e.result.entries.len()).unwrap_or(0),
            })
            .collect();
        let passed = files.iter().filter(|f| f.ok).count();
        let summary = RunSummary {
            id: id.clone(),
            label,
            started_at,
            duration_ms: started.elapsed().as_millis(),
            total,
            passed,
            failed: files.len() - passed,
            cancelled,
            files,
        };

        let mut stored = self.stored.lock().unwrap();
        stored.push_back(StoredRun { id, runs, executed });
        while stored.len() > MAX_STORED {
            stored.pop_front();
        }
        on_event(RunEvent::Done {
            summary: summary.clone(),
        });
        Ok(summary)
    }

    pub fn file_runs(&self, run_id: &str) -> Option<Vec<FileRun>> {
        self.stored
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == run_id)
            .map(|r| r.runs.clone())
    }

    /// Full (untruncated) body of one response, taken from the in-memory result.
    /// Raw body and `Content-Type` of one response, from the in-memory result.
    pub fn response_bytes(
        &self,
        run_id: &str,
        file: usize,
        entry: usize,
        call: usize,
    ) -> Result<(Vec<u8>, Option<String>), RunError> {
        let stored = self.stored.lock().unwrap();
        stored
            .iter()
            .find(|r| r.id == run_id)
            .and_then(|run| run.executed.get(file)?.as_ref())
            .and_then(|ex| ex.result.entries.get(entry)?.calls.get(call))
            .map(|c| {
                let ct = c
                    .response
                    .headers
                    .iter()
                    .find(|h| h.name.eq_ignore_ascii_case("content-type"))
                    .map(|h| h.value.clone());
                (c.response.body.clone(), ct)
            })
            .ok_or(RunError::NotInMemory)
    }

    pub fn response_body(
        &self,
        run_id: &str,
        file: usize,
        entry: usize,
        call: usize,
    ) -> Result<String, RunError> {
        let stored = self.stored.lock().unwrap();
        let body = stored
            .iter()
            .find(|r| r.id == run_id)
            .and_then(|run| run.executed.get(file)?.as_ref())
            .and_then(|ex| ex.result.entries.get(entry)?.calls.get(call))
            .map(|c| String::from_utf8_lossy(&c.response.body).to_string());
        body.ok_or(RunError::NotInMemory)
    }

    pub fn export(&self, run_id: &str, format: &str, dir: &Path) -> Result<PathBuf, RunError> {
        let stored = self.stored.lock().unwrap();
        let run = stored
            .iter()
            .find(|r| r.id == run_id)
            .ok_or(RunError::NotInMemory)?;
        std::fs::create_dir_all(dir).map_err(RunError::io("cannot create", dir))?;
        let items: Vec<&Executed> = run.executed.iter().flatten().collect();
        let secrets: Vec<&str> = items
            .iter()
            .flat_map(|e| e.secrets.iter().map(String::as_str))
            .collect();
        match format {
            "html" => {
                use ::hurl::report::html::{Testcase, write_report};
                let mut cases = Vec::new();
                for ex in &items {
                    let tc = Testcase::from(&ex.result, &ex.input);
                    tc.write_html(&ex.content, &ex.result.entries, dir, &secrets)
                        .map_err(RunError::report)?;
                    cases.push(tc);
                }
                write_report(dir, &cases).map_err(RunError::report)?;
                Ok(dir.join("index.html"))
            }
            "junit" => {
                use ::hurl::report::junit::{Testcase, write_report};
                let cases: Vec<Testcase> = items
                    .iter()
                    .map(|ex| Testcase::from(&ex.result, &ex.content, &ex.input))
                    .collect();
                let file = dir.join("junit.xml");
                let _ = std::fs::remove_file(&file);
                write_report(&file, &cases, &secrets).map_err(RunError::report)?;
                Ok(file)
            }
            "tap" => {
                use ::hurl::report::tap::{Testcase, write_report};
                let cases: Vec<Testcase> = items
                    .iter()
                    .map(|ex| Testcase::from(&ex.result, &ex.input))
                    .collect();
                let file = dir.join("report.tap");
                let _ = std::fs::remove_file(&file);
                write_report(&file, &cases).map_err(RunError::report)?;
                Ok(file)
            }
            "json" => {
                let file = dir.join("report.json");
                let text = serde_json::to_string_pretty(&run.runs)?;
                std::fs::write(&file, text).map_err(RunError::io("cannot write", &file))?;
                Ok(file)
            }
            other => Err(RunError::UnknownFormat(other.to_string())),
        }
    }
}

// ------------------------------------------------------------------ history

fn history_file(data_dir: &Path) -> PathBuf {
    data_dir.join("history.json")
}

pub fn list_history(data_dir: &Path) -> Vec<RunSummary> {
    std::fs::read_to_string(history_file(data_dir))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_history(data_dir: &Path, summary: &RunSummary, runs: &[FileRun]) {
    let _ = std::fs::create_dir_all(data_dir.join("runs"));
    let mut list = list_history(data_dir);
    list.insert(0, summary.clone());
    for old in list.drain(MAX_HISTORY.min(list.len())..) {
        let _ = std::fs::remove_file(data_dir.join("runs").join(format!("{}.json", old.id)));
    }
    if let Ok(t) = serde_json::to_string(&list) {
        let _ = std::fs::write(history_file(data_dir), t);
    }
    if let Ok(t) = serde_json::to_string(runs) {
        let _ = std::fs::write(
            data_dir.join("runs").join(format!("{}.json", summary.id)),
            t,
        );
    }
}

pub fn load_history_run(data_dir: &Path, id: &str) -> Result<serde_json::Value, RunError> {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(RunError::InvalidRunId);
    }
    let file = data_dir.join("runs").join(format!("{id}.json"));
    let text = std::fs::read_to_string(&file).map_err(RunError::io("cannot read", &file))?;
    Ok(serde_json::from_str(&text)?)
}

pub fn delete_history(data_dir: &Path) {
    let _ = std::fs::remove_dir_all(data_dir.join("runs"));
    let _ = std::fs::remove_file(history_file(data_dir));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn mock_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = r#"{"name":"gurl","n":3}"#;
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        port
    }

    #[test]
    fn dir_run_uses_inherited_vars_and_exports_reports() {
        let port = mock_server();
        let root = std::env::temp_dir().join(format!("gurl-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let sub = root.join("api").join("v1");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(
            root.join("gurl.env"),
            format!("host=127.0.0.1:{port}\nname=wrong\n"),
        )
        .unwrap();
        std::fs::write(root.join("api").join("gurl.env"), "name=gurl\n").unwrap();
        std::fs::write(
            sub.join("a.hurl"),
            "GET http://{{host}}/x\nHTTP 200\n[Asserts]\njsonpath \"$.name\" == \"{{name}}\"\njsonpath \"$.n\" == 3\n",
        )
        .unwrap();
        std::fs::write(sub.join("b.hurl"), "GET http://{{host}}/y\nHTTP 404\n").unwrap();

        let files = workspace::collect_hurl_files(&root, &root, true).unwrap();
        assert_eq!(files.len(), 2);

        let store = RunStore::default();
        // `Channel` can only be built by Tauri's IPC; exercise the pieces it wraps instead.
        let env = env_for(Some(&root), &files[0]);
        assert!(env.iter().any(|v| v.name == "name" && v.value == "gurl"));
        let opts = HurlRunOptions::default();
        let (run_a, ex_a) = hurl::run(Source::File(files[0].clone()), &opts, &env);
        assert!(run_a.ok, "{:?}", run_a.error);
        let asserts = &run_a.report.as_ref().unwrap()["entries"][0]["asserts"];
        assert!(asserts.as_array().unwrap().iter().any(|a| {
            a["source"]
                .as_str()
                .is_some_and(|s| s.starts_with("jsonpath \"$.n\""))
        }));
        let (run_b, ex_b) = hurl::run(Source::File(files[1].clone()), &opts, &env);
        assert!(!run_b.ok);

        store.stored.lock().unwrap().push_back(StoredRun {
            id: "t".into(),
            runs: vec![run_a, run_b],
            executed: vec![ex_a, ex_b],
        });
        for (fmt, file) in [
            ("html", "index.html"),
            ("junit", "junit.xml"),
            ("tap", "report.tap"),
            ("json", "report.json"),
        ] {
            let out = root.join(format!("out-{fmt}"));
            let path = store.export("t", fmt, &out).unwrap();
            assert!(path.ends_with(file), "{path:?}");
            assert!(path.exists());
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
