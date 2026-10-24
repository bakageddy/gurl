// Data tables that drive the builder forms. Everything the HURL grammar offers for
// queries, filters, predicates and entry options lives here.

export const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "CONNECT", "TRACE"];

export const VERSIONS = [
  { value: "HTTP", label: "HTTP (any)" },
  { value: "HTTP/1.0", label: "HTTP/1.0" },
  { value: "HTTP/1.1", label: "HTTP/1.1" },
  { value: "HTTP/2", label: "HTTP/2" },
  { value: "HTTP/3", label: "HTTP/3" },
];

export const COMMON_STATUS = ["*", "200", "201", "202", "204", "301", "302", "304", "400", "401", "403", "404", "409", "422", "429", "500", "502", "503"];

export interface QueryKind {
  kind: string;
  label: string;
  /** Placeholder for the argument box; `null` when the query takes no argument. */
  arg: string | null;
  group: "response" | "body" | "meta";
}

export const QUERY_KINDS: QueryKind[] = [
  { kind: "status", label: "status", arg: null, group: "response" },
  { kind: "version", label: "version", arg: null, group: "response" },
  { kind: "url", label: "url", arg: null, group: "response" },
  { kind: "header", label: "header", arg: '"Content-Type"', group: "response" },
  { kind: "cookie", label: "cookie", arg: '"name[Secure]"', group: "response" },
  { kind: "ip", label: "ip", arg: null, group: "response" },
  { kind: "redirects", label: "redirects", arg: null, group: "response" },
  { kind: "certificate", label: "certificate", arg: '"Subject"', group: "response" },
  { kind: "body", label: "body", arg: null, group: "body" },
  { kind: "jsonpath", label: "jsonpath", arg: '"$.items[0].id"', group: "body" },
  { kind: "xpath", label: "xpath", arg: '"string(//title)"', group: "body" },
  { kind: "regex", label: "regex", arg: '/id=(\\d+)/', group: "body" },
  { kind: "bytes", label: "bytes", arg: null, group: "body" },
  { kind: "rawbytes", label: "rawbytes", arg: null, group: "body" },
  { kind: "sha256", label: "sha256", arg: null, group: "body" },
  { kind: "md5", label: "md5", arg: null, group: "body" },
  { kind: "variable", label: "variable", arg: '"name"', group: "meta" },
  { kind: "duration", label: "duration", arg: null, group: "meta" },
];

export const QUERY_GROUPS: Array<{ id: QueryKind["group"]; label: string }> = [
  { id: "response", label: "Response" },
  { id: "body", label: "Body" },
  { id: "meta", label: "Other" },
];

export function queryKind(kind: string): QueryKind | undefined {
  return QUERY_KINDS.find((q) => q.kind === kind);
}

export interface FilterDef {
  name: string;
  arg: string | null;
}

export const FILTERS: FilterDef[] = [
  { name: "base64Decode", arg: null },
  { name: "base64Encode", arg: null },
  { name: "base64UrlSafeDecode", arg: null },
  { name: "base64UrlSafeEncode", arg: null },
  { name: "charsetDecode", arg: '"utf-8"' },
  { name: "count", arg: null },
  { name: "daysAfterNow", arg: null },
  { name: "daysBeforeNow", arg: null },
  { name: "decode", arg: '"utf-8"' },
  { name: "first", arg: null },
  { name: "format", arg: '"%Y-%m-%d"' },
  { name: "dateFormat", arg: '"%Y-%m-%d"' },
  { name: "htmlEscape", arg: null },
  { name: "htmlUnescape", arg: null },
  { name: "jsonpath", arg: '"$.a"' },
  { name: "last", arg: null },
  { name: "location", arg: null },
  { name: "nth", arg: "0" },
  { name: "regex", arg: '/(\\d+)/' },
  { name: "replace", arg: '"old" "new"' },
  { name: "replaceRegex", arg: '/a+/ "b"' },
  { name: "split", arg: '","' },
  { name: "toDate", arg: '"%Y-%m-%d"' },
  { name: "toFloat", arg: null },
  { name: "toHex", arg: null },
  { name: "toInt", arg: null },
  { name: "toString", arg: null },
  { name: "urlDecode", arg: null },
  { name: "urlEncode", arg: null },
  { name: "urlQueryParam", arg: '"id"' },
  { name: "utf8Decode", arg: null },
  { name: "utf8Encode", arg: null },
  { name: "xpath", arg: '"string(//a)"' },
];

export function filterDef(name: string): FilterDef | undefined {
  return FILTERS.find((f) => f.name === name);
}

export interface PredicateDef {
  func: string;
  label: string;
  /** Whether the predicate takes a value. */
  value: boolean;
}

export const PREDICATES: PredicateDef[] = [
  { func: "==", label: "equals", value: true },
  { func: "!=", label: "not equals", value: true },
  { func: ">", label: "greater than", value: true },
  { func: ">=", label: "greater or equal", value: true },
  { func: "<", label: "less than", value: true },
  { func: "<=", label: "less or equal", value: true },
  { func: "startsWith", label: "starts with", value: true },
  { func: "endsWith", label: "ends with", value: true },
  { func: "contains", label: "contains", value: true },
  { func: "includes", label: "includes", value: true },
  { func: "matches", label: "matches regex", value: true },
  { func: "exists", label: "exists", value: false },
  { func: "isBoolean", label: "is boolean", value: false },
  { func: "isCollection", label: "is collection", value: false },
  { func: "isDate", label: "is date", value: false },
  { func: "isEmpty", label: "is empty", value: false },
  { func: "isFloat", label: "is float", value: false },
  { func: "isInteger", label: "is integer", value: false },
  { func: "isIpv4", label: "is IPv4", value: false },
  { func: "isIpv6", label: "is IPv6", value: false },
  { func: "isIsoDate", label: "is ISO date", value: false },
  { func: "isList", label: "is list", value: false },
  { func: "isNumber", label: "is number", value: false },
  { func: "isObject", label: "is object", value: false },
  { func: "isString", label: "is string", value: false },
  { func: "isUuid", label: "is UUID", value: false },
];

export function predicateDef(func: string): PredicateDef | undefined {
  return PREDICATES.find((p) => p.func === func);
}

export type OptionType = "bool" | "text" | "duration" | "number" | "file" | "verbosity";
export interface OptionDef {
  name: string;
  type: OptionType;
  label: string;
  group: string;
  hint?: string;
}

export const OPTION_DEFS: OptionDef[] = [
  { name: "location", type: "bool", label: "Follow redirects", group: "Redirects" },
  { name: "location-trusted", type: "bool", label: "Follow redirects (trusted)", group: "Redirects" },
  { name: "max-redirs", type: "number", label: "Max redirects", group: "Redirects", hint: "50" },
  { name: "retry", type: "number", label: "Retry count", group: "Flow", hint: "3" },
  { name: "retry-interval", type: "duration", label: "Retry interval", group: "Flow", hint: "1000ms" },
  { name: "delay", type: "duration", label: "Delay before request", group: "Flow", hint: "500ms" },
  { name: "repeat", type: "number", label: "Repeat", group: "Flow", hint: "2" },
  { name: "skip", type: "bool", label: "Skip entry", group: "Flow" },
  { name: "variable", type: "text", label: "Variable", group: "Flow", hint: "name=value" },
  { name: "connect-timeout", type: "duration", label: "Connect timeout", group: "Timeouts", hint: "10s" },
  { name: "max-time", type: "duration", label: "Max time", group: "Timeouts", hint: "30s" },
  { name: "insecure", type: "bool", label: "Skip TLS verification", group: "TLS" },
  { name: "cacert", type: "file", label: "CA certificate", group: "TLS" },
  { name: "cert", type: "file", label: "Client certificate", group: "TLS" },
  { name: "key", type: "file", label: "Client key", group: "TLS" },
  { name: "pinnedpubkey", type: "text", label: "Pinned public key", group: "TLS" },
  { name: "compressed", type: "bool", label: "Request compression", group: "HTTP" },
  { name: "http1.0", type: "bool", label: "Force HTTP/1.0", group: "HTTP" },
  { name: "http1.1", type: "bool", label: "Force HTTP/1.1", group: "HTTP" },
  { name: "http2", type: "bool", label: "Force HTTP/2", group: "HTTP" },
  { name: "http3", type: "bool", label: "Force HTTP/3", group: "HTTP" },
  { name: "ipv4", type: "bool", label: "IPv4 only", group: "HTTP" },
  { name: "ipv6", type: "bool", label: "IPv6 only", group: "HTTP" },
  { name: "path-as-is", type: "bool", label: "Keep path as is", group: "HTTP" },
  { name: "header", type: "text", label: "Extra header", group: "HTTP", hint: "Name: value" },
  { name: "output", type: "file", label: "Write body to file", group: "HTTP" },
  { name: "limit-rate", type: "number", label: "Rate limit (bytes/s)", group: "HTTP" },
  { name: "resolve", type: "text", label: "Resolve", group: "Network", hint: "host:port:addr" },
  { name: "connect-to", type: "text", label: "Connect to", group: "Network", hint: "h1:p1:h2:p2" },
  { name: "proxy", type: "text", label: "Proxy", group: "Network", hint: "http://host:3128" },
  { name: "unix-socket", type: "file", label: "Unix socket", group: "Network" },
  { name: "user", type: "text", label: "User", group: "Auth", hint: "user:password" },
  { name: "digest", type: "bool", label: "Digest auth", group: "Auth" },
  { name: "negotiate", type: "bool", label: "Negotiate auth", group: "Auth" },
  { name: "ntlm", type: "bool", label: "NTLM auth", group: "Auth" },
  { name: "aws-sigv4", type: "text", label: "AWS SigV4", group: "Auth", hint: "aws:amz:region:service" },
  { name: "netrc", type: "bool", label: ".netrc", group: "Auth" },
  { name: "netrc-file", type: "file", label: ".netrc file", group: "Auth" },
  { name: "netrc-optional", type: "bool", label: ".netrc optional", group: "Auth" },
  { name: "verbose", type: "bool", label: "Verbose", group: "Logging" },
  { name: "very-verbose", type: "bool", label: "Very verbose", group: "Logging" },
  { name: "verbosity", type: "verbosity", label: "Verbosity", group: "Logging" },
];

export function optionDef(name: string): OptionDef | undefined {
  return OPTION_DEFS.find((o) => o.name === name);
}

export interface BodyKindDef {
  kind: string;
  label: string;
  hint: string;
  mono: boolean;
  /** Language used by the CodeMirror body editor. */
  lang: "json" | "xml" | "plain";
}

export const BODY_KINDS: BodyKindDef[] = [
  { kind: "json", label: "JSON", hint: '{ "key": "{{value}}" }', mono: true, lang: "json" },
  { kind: "multiline-json", label: "JSON (multiline)", hint: '{\n  "key": "value"\n}', mono: true, lang: "json" },
  { kind: "xml", label: "XML", hint: "<root/>", mono: true, lang: "xml" },
  { kind: "multiline-xml", label: "XML (multiline)", hint: "<root>\n</root>", mono: true, lang: "xml" },
  { kind: "text", label: "Text (one line)", hint: "hello", mono: true, lang: "plain" },
  { kind: "multiline", label: "Text (multiline)", hint: "line 1\nline 2", mono: true, lang: "plain" },
  { kind: "multiline-raw", label: "Raw (multiline)", hint: "raw text", mono: true, lang: "plain" },
  { kind: "graphql", label: "GraphQL", hint: "query { me { id } }", mono: true, lang: "plain" },
  { kind: "base64", label: "Base64", hint: "SGVsbG8=", mono: true, lang: "plain" },
  { kind: "hex", label: "Hex", hint: "48656c6c6f", mono: true, lang: "plain" },
  { kind: "file", label: "File", hint: "data.json", mono: true, lang: "plain" },
];

export const STATUS_TEXT: Record<number, string> = {
  200: "OK", 201: "Created", 202: "Accepted", 204: "No Content", 301: "Moved Permanently",
  302: "Found", 304: "Not Modified", 400: "Bad Request", 401: "Unauthorized", 403: "Forbidden",
  404: "Not Found", 405: "Method Not Allowed", 409: "Conflict", 422: "Unprocessable Entity",
  429: "Too Many Requests", 500: "Internal Server Error", 502: "Bad Gateway", 503: "Service Unavailable",
};

export function newEntry(): import("./types").EntryM {
  return {
    comments: [],
    request: {
      method: "GET",
      url: "https://example.org",
      headers: [],
      query: [],
      form: [],
      multipart: [],
      cookies: [],
      basicAuth: null,
      options: [],
      body: null,
    },
    response: { version: "HTTP", status: "200", headers: [], captures: [], asserts: [], body: null },
  };
}
