// Mirrors src-tauri/src/model.rs, workspace.rs, hurl.rs and runs.rs (camelCase JSON).

export interface Kv {
  name: string;
  value: string;
}
export interface MultipartM {
  name: string;
  kind: "param" | "file";
  value: string;
  contentType: string | null;
}
export type BodyKind =
  | "json"
  | "xml"
  | "text"
  | "multiline"
  | "multiline-json"
  | "multiline-xml"
  | "multiline-raw"
  | "graphql"
  | "base64"
  | "hex"
  | "file";
export interface BodyM {
  kind: BodyKind;
  text: string;
}
export interface QueryM {
  kind: string;
  args: string;
}
export interface FilterM {
  name: string;
  args: string;
}
export interface CaptureM {
  name: string;
  query: QueryM;
  filters: FilterM[];
  redacted: boolean;
}
export interface AssertM {
  query: QueryM;
  filters: FilterM[];
  not: boolean;
  func: string;
  value: string;
}
export interface RequestM {
  method: string;
  url: string;
  headers: Kv[];
  query: Kv[];
  form: Kv[];
  multipart: MultipartM[];
  cookies: Kv[];
  basicAuth: Kv | null;
  options: Kv[];
  body: BodyM | null;
}
export interface ResponseM {
  version: string;
  status: string;
  headers: Kv[];
  captures: CaptureM[];
  asserts: AssertM[];
  body: BodyM | null;
}
export interface EntryM {
  comments: string[];
  request: RequestM;
  response: ResponseM | null;
}
export interface HurlDoc {
  entries: EntryM[];
  trailingComments: string[];
}
export interface Diagnostic {
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  message: string;
}
export interface ParseOutcome {
  model: HurlDoc | null;
  diagnostics: Diagnostic[];
  lossless: boolean;
  droppedComments: number;
}

export interface TreeNode {
  name: string;
  path: string;
  kind: "dir" | "hurl" | "env";
  hasEnv: boolean;
}
export interface WorkspaceInfo {
  root: string;
  maxDepth: number;
  nodes: TreeNode[];
}
export interface EnvVar {
  name: string;
  value: string;
  secret: boolean;
}
export interface EffectiveVar extends EnvVar {
  sourceDir: string;
  overrides: string[];
}

export interface RunOptions {
  variables: Record<string, string>;
  insecure: boolean;
  follow_location: boolean;
  follow_location_trusted: boolean;
  max_redirect: number;
  compressed: boolean;
  timeout_ms: number;
  connect_timeout_ms: number;
  retry: number;
  retry_interval_ms: number;
  delay_ms: number;
  http_version: string;
  ip_resolve: string;
  user: string;
  user_agent: string;
  proxy: string;
  no_proxy: string;
  cacert_file: string;
  client_cert_file: string;
  client_key_file: string;
  fail_fast: boolean;
  no_assert: boolean;
  use_cookie_store: boolean;
  verbosity: string;
  repeat: number;
}

export const DEFAULT_RUN_OPTIONS: RunOptions = {
  variables: {},
  insecure: false,
  follow_location: false,
  follow_location_trusted: false,
  max_redirect: 50,
  compressed: false,
  timeout_ms: 300_000,
  connect_timeout_ms: 300_000,
  retry: 0,
  retry_interval_ms: 1000,
  delay_ms: 0,
  http_version: "",
  ip_resolve: "",
  user: "",
  user_agent: "",
  proxy: "",
  no_proxy: "",
  cacert_file: "",
  client_cert_file: "",
  client_key_file: "",
  fail_fast: true,
  no_assert: false,
  use_cookie_store: true,
  verbosity: "",
  repeat: 1,
};

export interface HurlHeader {
  name: string;
  value: string;
}
export interface HurlCall {
  request: {
    method: string;
    url: string;
    headers?: HurlHeader[];
    query_string?: HurlHeader[];
    cookies?: HurlHeader[];
  };
  response: {
    status: number;
    http_version?: string;
    headers?: HurlHeader[];
    cookies?: Array<{ name: string; value: string }>;
    certificate?: Record<string, string | null> | null;
    body?: string;
    bodySize?: number;
    bodyTruncated?: boolean;
  };
  timings?: {
    total: number;
    name_lookup: number;
    connect: number;
    app_connect: number;
    pre_transfer: number;
    start_transfer: number;
  };
}
export interface HurlEntryReport {
  index: number;
  line: number;
  asserts: Array<{ success: boolean; line: number; message?: string }>;
  captures: Array<{ name: string; value: unknown }>;
  calls: HurlCall[];
  time: number;
  curl_cmd: string;
}
export interface HurlReport {
  filename: string;
  success: boolean;
  time: number;
  entries: HurlEntryReport[];
}
export interface FileRun {
  path: string;
  ok: boolean;
  durationMs: number;
  report: HurlReport | null;
  errors: string[];
  error: string | null;
}
export interface FileSummary {
  path: string;
  ok: boolean;
  durationMs: number;
  entries: number;
}
export interface RunSummary {
  id: string;
  label: string;
  startedAt: number;
  durationMs: number;
  total: number;
  passed: number;
  failed: number;
  cancelled: boolean;
  files: FileSummary[];
}
export type RunEvent =
  | { event: "started"; runId: string; files: string[] }
  | { event: "fileStarted"; index: number }
  | { event: "fileFinished"; index: number; run: FileRun }
  | { event: "done"; summary: RunSummary };

export type ScopeKind = "text" | "file" | "dir";
export type ReportFormat = "html" | "junit" | "tap" | "json";
