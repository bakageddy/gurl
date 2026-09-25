export interface HurlRunOptions {
  variables: Record<string, string>;

  insecure: boolean;
  followLocation: boolean;
  followLocationTrusted: boolean;
  maxRedirect: number;

  compressed: boolean;
  timeoutMs: number;
  connectTimeoutMs: number;
  retry: number;
  retryIntervalMs: number;
  delayMs: number;

  httpVersion: string; // "default" | "1.0" | "1.1" | "2" | "3"
  ipResolve: string; // "default" | "v4" | "v6"

  user: string;
  userAgent: string;
  proxy: string;
  noProxy: string;

  cacertFile: string;
  clientCertFile: string;
  clientKeyFile: string;

  fromEntry: number | null;
  toEntry: number | null;

  failFast: boolean;
  noAssert: boolean;
  skip: boolean;
  useCookieStore: boolean;

  resolves: string;
  connectsTo: string;

  verbosity: string; // "none" | "verbose" | "very-verbose"

  // Authentication methods
  digest: boolean;
  negotiate: boolean;
  ntlm: boolean;
  netrc: boolean;
  netrcOptional: boolean;
  netrcFile: string;
  awsSigv4: string;

  // TLS / connection
  sslNoRevoke: boolean;
  pathAsIs: boolean;
  pinnedPubKey: string;
  unixSocket: string;

  // Transfer speed / repeat
  maxSendSpeed: number | null;
  maxRecvSpeed: number | null;
  repeat: number;

  allowReuse: boolean;
}

export const DEFAULT_OPTIONS: HurlRunOptions = {
  variables: {},
  insecure: false,
  followLocation: false,
  followLocationTrusted: false,
  maxRedirect: 50,
  compressed: false,
  timeoutMs: 300_000,
  connectTimeoutMs: 300_000,
  retry: 0,
  retryIntervalMs: 1000,
  delayMs: 0,
  httpVersion: "default",
  ipResolve: "default",
  user: "",
  userAgent: "zurl/0.1.0",
  proxy: "",
  noProxy: "",
  cacertFile: "",
  clientCertFile: "",
  clientKeyFile: "",
  fromEntry: null,
  toEntry: null,
  failFast: true,
  noAssert: false,
  skip: false,
  useCookieStore: true,
  resolves: "",
  connectsTo: "",
  verbosity: "none",

  digest: false,
  negotiate: false,
  ntlm: false,
  netrc: false,
  netrcOptional: false,
  netrcFile: "",
  awsSigv4: "",

  sslNoRevoke: false,
  pathAsIs: false,
  pinnedPubKey: "",
  unixSocket: "",

  maxSendSpeed: null,
  maxRecvSpeed: null,
  repeat: 1,

  allowReuse: true,
};

export interface KeyValue {
  id: string;
  name: string;
  value: string;
  enabled: boolean;
}

export function newKeyValue(partial: Partial<KeyValue> = {}): KeyValue {
  return {
    id: Math.random().toString(36).slice(2),
    name: "",
    value: "",
    enabled: true,
    ...partial,
  };
}

export interface QuickRequest {
  id: string;
  name: string;
  method: string;
  url: string;
  params: KeyValue[];
  headers: KeyValue[];
  body: string;
  rawMode: boolean;
  options: HurlRunOptions;
}

export function newQuickRequest(name = "New request"): QuickRequest {
  return {
    id: Math.random().toString(36).slice(2),
    name,
    method: "GET",
    url: "https://httpbin.org/get",
    params: [],
    headers: [newKeyValue({ name: "User-Agent", value: "zurl/0.1.0" })],
    body: "",
    rawMode: false,
    options: { ...DEFAULT_OPTIONS },
  };
}

export interface FileRequest {
  id: string;
  path: string;
  name: string;
  content: string;
  dirty: boolean;
}

export type Tab =
  | { kind: "quick"; req: QuickRequest }
  | { kind: "file"; file: FileRequest };

export interface TreeEntry {
  name: string;
  path: string;
  is_dir: boolean;
  children: TreeEntry[];
}

export interface HurlCall {
  request: {
    method: string;
    url: string;
    headers?: Array<{ name: string; value: string }>;
    query_string?: Array<{ name: string; value: string }>;
    cookies?: Array<{ name: string; value: string }>;
  };
  response: {
    status: number;
    http_version?: string;
    headers?: Array<{ name: string; value: string }>;
    cookies?: Array<{ name: string; value: string; expires?: string | null }>;
    certificate?: Record<string, string | null>;
    body?: string;
  };
  timings?: {
    total: number;
    name_lookup: number;
    connect: number;
    app_connect: number;
    pre_transfer: number;
    start_transfer: number;
    begin_call: string;
    end_call: string;
  };
}

export interface HurlEntry {
  index: number;
  line: number;
  asserts: Array<{ success: boolean; line: number; message?: string }>;
  captures: Array<{ name: string; value: unknown }>;
  calls: HurlCall[];
  time: number;
  curl_cmd: string;
}

export interface HurlReportFile {
  filename: string;
  success: boolean;
  time: number;
  cookies: unknown[];
  entries: HurlEntry[];
}

export interface RunResult {
  ok: boolean;
  duration_ms: number;
  report?: HurlReportFile;
  errors: string[];
  error?: string;
}

export interface Workspace {
  path: string;
  tree: TreeEntry[];
}
