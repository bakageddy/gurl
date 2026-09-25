import { invoke } from "@tauri-apps/api/core";
import type { DirEntry, HurlRunOptions, RunResult, TreeEntry } from "../types";

function snakeCaseOpts(opts: HurlRunOptions): Record<string, unknown> {
  return {
    variables: opts.variables,
    insecure: opts.insecure,
    follow_location: opts.followLocation,
    follow_location_trusted: opts.followLocationTrusted,
    max_redirect: opts.maxRedirect,
    compressed: opts.compressed,
    timeout_ms: opts.timeoutMs,
    connect_timeout_ms: opts.connectTimeoutMs,
    retry: opts.retry,
    retry_interval_ms: opts.retryIntervalMs,
    delay_ms: opts.delayMs,
    http_version: opts.httpVersion,
    ip_resolve: opts.ipResolve,
    user: opts.user,
    user_agent: opts.userAgent,
    proxy: opts.proxy,
    no_proxy: opts.noProxy,
    cacert_file: opts.cacertFile,
    client_cert_file: opts.clientCertFile,
    client_key_file: opts.clientKeyFile,
    from_entry: opts.fromEntry,
    to_entry: opts.toEntry,
    fail_fast: opts.failFast,
    no_assert: opts.noAssert,
    skip: opts.skip,
    use_cookie_store: opts.useCookieStore,
    resolves: opts.resolves.split("\n").map((s) => s.trim()).filter(Boolean),
    connects_to: opts.connectsTo.split("\n").map((s) => s.trim()).filter(Boolean),
    verbosity: opts.verbosity,

    digest: opts.digest,
    negotiate: opts.negotiate,
    ntlm: opts.ntlm,
    netrc: opts.netrc,
    netrc_optional: opts.netrcOptional,
    netrc_file: opts.netrcFile,
    aws_sigv4: opts.awsSigv4,

    ssl_no_revoke: opts.sslNoRevoke,
    path_as_is: opts.pathAsIs,
    pinned_pub_key: opts.pinnedPubKey,
    unix_socket: opts.unixSocket,

    max_send_speed: opts.maxSendSpeed,
    max_recv_speed: opts.maxRecvSpeed,
    repeat: opts.repeat,

    allow_reuse: opts.allowReuse,
  };
}

export function hurlVersion(): Promise<string> {
  return invoke("hurl_version");
}

export function runHurlFile(
  path: string,
  options: HurlRunOptions
): Promise<RunResult> {
  return invoke("run_hurl_file", { path, options: snakeCaseOpts(options) });
}

export function runHurlText(
  content: string,
  options: HurlRunOptions
): Promise<RunResult> {
  return invoke("run_hurl_text", {
    content,
    options: snakeCaseOpts(options),
  });
}

export function openFileDialog(): Promise<string | null> {
  return invoke("open_file_dialog");
}

export function openFolderDialog(): Promise<string | null> {
  return invoke("open_folder_dialog");
}

export function readTextFile(path: string): Promise<string> {
  return invoke("read_text_file", { path });
}

export function writeTextFile(path: string, content: string): Promise<void> {
  return invoke("write_text_file", { request: { path, content } });
}

export function listDirectory(path: string): Promise<DirEntry[]> {
  return invoke("list_directory", { path });
}

export function listDirectoryTree(path: string): Promise<TreeEntry[]> {
  return invoke("list_directory_tree", { path });
}
