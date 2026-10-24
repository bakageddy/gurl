import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AssertM,
  CaptureM,
  EffectiveVar,
  EnvVar,
  FileRun,
  HurlDoc,
  ParseOutcome,
  ReportFormat,
  RunEvent,
  RunOptions,
  RunSummary,
  ScopeKind,
  TreeNode,
  WorkspaceInfo,
} from "./types";

export const hurlVersion = () => invoke<string>("hurl_version");
export const parseHurl = (text: string) => invoke<ParseOutcome>("parse_hurl", { text });
export const emitHurl = (model: HurlDoc) => invoke<string>("emit_hurl", { model });
export const formatHurl = (text: string) => invoke<string>("format_hurl", { text });

export const openWorkspace = (path: string) => invoke<WorkspaceInfo>("open_workspace", { path });
export const closeWorkspace = () => invoke<void>("close_workspace");
export const listDir = (path: string) => invoke<TreeNode[]>("list_dir", { path });
export const createFile = (dir: string, name: string, content?: string) =>
  invoke<string>("create_file", { dir, name, content });
export const createDir = (parent: string, name: string) =>
  invoke<string>("create_dir", { parent, name });
export const renamePath = (path: string, newName: string) =>
  invoke<string>("rename_path", { path, newName });
export const deletePath = (path: string) => invoke<void>("delete_path", { path });
export const duplicatePath = (path: string) => invoke<string>("duplicate_path", { path });
export const getDirVars = (dir: string) => invoke<EnvVar[]>("get_dir_vars", { dir });
export const setDirVars = (dir: string, vars: EnvVar[]) =>
  invoke<void>("set_dir_vars", { dir, vars });
export const getEnvText = (dir: string) => invoke<string>("get_env_text", { dir });
export const setEnvText = (dir: string, text: string) =>
  invoke<void>("set_env_text", { dir, text });
export const getEffectiveVars = (dir: string) =>
  invoke<EffectiveVar[]>("get_effective_vars", { dir });

export const readTextFile = (path: string) => invoke<string>("read_text_file", { path });
export const writeTextFile = (path: string, content: string) =>
  invoke<void>("write_text_file", { path, content });
export const openFileDialog = () => invoke<string | null>("open_file_dialog");
export const openFolderDialog = () => invoke<string | null>("open_folder_dialog");

export interface ScopeRequest {
  kind: ScopeKind;
  path?: string;
  content?: string;
  recursive?: boolean;
  jobs?: number;
  options: RunOptions;
}

export function runScope(request: ScopeRequest, onEvent: (e: RunEvent) => void) {
  const channel = new Channel<RunEvent>();
  channel.onmessage = onEvent;
  return invoke<RunSummary>("run_scope", { request, onEvent: channel });
}
export const cancelRun = (runId: string) => invoke<void>("cancel_run", { runId });
export const getResponseBody = (runId: string, file: number, entry: number, call: number) =>
  invoke<string>("get_response_body", { runId, file, entry, call });

export interface BodyBytes {
  base64: string;
  contentType: string | null;
  truncated: boolean;
}
export const getResponseBodyBytes = (runId: string, file: number, entry: number, call: number) =>
  invoke<BodyBytes>("get_response_body_bytes", { runId, file, entry, call });

export type QueryKind = "jsonpath" | "xpath";
/** What hurl's own evaluator returns for a query; `kind` drives rendering and predicate choice. */
export interface QueryOutcome {
  kind: "none" | "single" | "list" | "number" | "bool" | "string" | "nodeset" | "error";
  value: unknown;
  /** `value` as JSON text with numbers exactly as the body spelled them. */
  text: string;
  count: number;
  error: string | null;
}
export const queryBody = (runId: string, file: number, entry: number, call: number, kind: QueryKind, expr: string) =>
  invoke<QueryOutcome>("query_body", { runId, file, entry, call, kind, expr });
export const queryText = (text: string, contentType: string | null, kind: QueryKind, expr: string) =>
  invoke<QueryOutcome>("query_text", { text, contentType, kind, expr });

export interface TextEdit {
  text: string;
  firstLine: number;
  added: number;
}
export const addResponseItems = (text: string, entry: number, asserts: AssertM[], captures: CaptureM[]) =>
  invoke<TextEdit>("add_response_items", { text, entry, asserts, captures });
export const listRuns = () => invoke<RunSummary[]>("list_runs");
export const loadRun = (runId: string) => invoke<FileRun[]>("load_run", { runId });
export const clearRuns = () => invoke<void>("clear_runs");
export const exportReport = (
  runId: string,
  format: ReportFormat,
  open: boolean,
  dir?: string,
) => invoke<string>("export_report", { runId, format, dir, open });

export const getSettings = () => invoke<Record<string, unknown> | null>("get_settings");
export const setSettings = (settings: Record<string, unknown>) =>
  invoke<void>("set_settings", { settings });

export interface DataFileStatus {
  exists: boolean;
  allowed: boolean;
}
/** Picks a data file inside the scenario's directory; returns a path relative to the .hurl file. */
export const pickDataFile = (hurlPath: string) =>
  invoke<string | null>("pick_data_file", { hurlPath });
export const checkDataFile = (hurlPath: string, reference: string) =>
  invoke<DataFileStatus>("check_data_file", { hurlPath, reference });
