import { createMemo, createSignal, For, onMount, Show } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import {
  FolderOpen,
  FileText,
  Play,
  Plus,
  RefreshCw,
  Save,
  X,
} from "lucide-solid";
import type {
  FileRequest,
  QuickRequest,
  RunResult,
  Tab,
  Workspace,
} from "./types";
import { DEFAULT_OPTIONS, newQuickRequest } from "./types";
import {
  hurlVersion,
  listDirectoryTree,
  openFileDialog,
  openFolderDialog,
  readTextFile,
  runHurlFile,
  runHurlText,
  writeTextFile,
} from "./lib/api";
import { RequestPanel } from "./components/RequestPanel";
import { ResponsePanel } from "./components/ResponsePanel";
import { Sidebar } from "./components/Sidebar";
import "./App.css";

function tabId(tab: Tab): string {
  return tab.kind === "quick" ? tab.req.id : tab.file.path;
}

function tabName(tab: Tab): string {
  if (tab.kind === "quick") return tab.req.name;
  const n = tab.file.name || tab.file.path;
  return tab.file.dirty ? `${n} ●` : n;
}

function buildHurlText(req: QuickRequest): string {
  if (req.rawMode) return req.body;
  const method = req.method === "CUSTOM" ? "GET" : req.method;
  const pairs = req.params
    .filter((p) => p.enabled && p.name)
    .map((p) => `${encodeURIComponent(p.name)}=${encodeURIComponent(p.value)}`);
  const url =
    req.url +
    (pairs.length
      ? `${req.url.includes("?") ? "&" : "?"}${pairs.join("&")}`
      : "");
  const headerLines = req.headers
    .filter((h) => h.enabled && h.name)
    .map((h) => `${h.name}: ${h.value}`);
  const parts = [`${method} ${url}`, ...headerLines];
  if (req.body) {
    parts.push("");
    parts.push(req.body);
  }
  return parts.join("\n");
}

export default function App() {
  const [version, setVersion] = createSignal<string>("");
  const [workspace, setWorkspace] = createSignal<Workspace | null>(null);
  const [tabs, setTabs] = createStore<Tab[]>([
    { kind: "quick", req: newQuickRequest() },
  ]);
  const [activeTabId, setActiveTabId] = createSignal<string>(tabId(tabs[0]));
  const [result, setResult] = createSignal<RunResult | null>(null);
  const [loading, setLoading] = createSignal(false);

  const activeTab = createMemo<Tab | undefined>(() =>
    tabs.find((t) => tabId(t) === activeTabId())
  );

  onMount(async () => {
    try {
      setVersion(await hurlVersion());
    } catch {
      setVersion("hurl crate ready");
    }
  });

  function addQuickTab(req = newQuickRequest()) {
    const tab: Tab = { kind: "quick", req };
    setTabs(tabs.length, tab);
    setActiveTabId(tabId(tab));
  }

  async function loadWorkspace(path: string) {
    const tree = await listDirectoryTree(path);
    setWorkspace({ path, tree });
  }

  async function openFolder() {
    const path = await openFolderDialog();
    if (path) await loadWorkspace(path);
  }

  async function openFile() {
    const path = await openFileDialog();
    if (path) await openFileInTab(path);
  }

  async function openFileInTab(path: string) {
    const existing = tabs.find(
      (t) => t.kind === "file" && t.file.path === path
    );
    if (existing) {
      setActiveTabId(tabId(existing));
      return;
    }
    const content = await readTextFile(path);
    const name = path.replace(/\\/g, "/").split("/").pop() || path;
    const tab: Tab = {
      kind: "file",
      file: { id: path, path, name, content, dirty: false },
    };
    setTabs(tabs.length, tab);
    setActiveTabId(path);
  }

  function closeTab(id: string) {
    const idx = tabs.findIndex((t) => tabId(t) === id);
    if (idx < 0) return;
    const next = tabs[idx + 1] || tabs[idx - 1];
    const remaining = tabs.filter((t) => tabId(t) !== id);
    setTabs(reconcile(remaining));
    if (activeTabId() === id) {
      setActiveTabId(next ? tabId(next) : "");
    }
  }

  async function runActiveTab() {
    const tab = activeTab();
    if (!tab) return;
    setLoading(true);
    setResult(null);
    try {
      let res: RunResult;
      if (tab.kind === "quick") {
        const text = buildHurlText(tab.req);
        res = await runHurlText(text, tab.req.options);
      } else {
        res = await runHurlFile(tab.file.path, DEFAULT_OPTIONS);
      }
      setResult(res);
    } catch (err: any) {
      setResult({
        ok: false,
        duration_ms: 0,
        errors: [String(err)],
        error: String(err),
      });
    } finally {
      setLoading(false);
    }
  }

  async function saveActiveFile() {
    const tab = activeTab();
    if (!tab || tab.kind !== "file") return;
    await writeTextFile(tab.file.path, tab.file.content);
    updateFileTab(tab.file.path, { dirty: false });
  }

  function updateQuickTab(id: string, patch: Partial<QuickRequest>) {
    const idx = tabs.findIndex((t) => t.kind === "quick" && t.req.id === id);
    if (idx < 0) return;
    setTabs(idx, "req", (req) => ({ ...req, ...patch }));
  }

  function updateFileTab(path: string, patch: Partial<FileRequest>) {
    const idx = tabs.findIndex(
      (t) => t.kind === "file" && t.file.path === path
    );
    if (idx < 0) return;
    setTabs(idx, "file", (file) => ({ ...file, ...patch }));
  }

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-bg text-text">
      <Sidebar
        workspace={workspace()}
        activePath={
          activeTab()?.kind === "file" ? activeTab()!.file.path : undefined
        }
        onOpenFolder={openFolder}
        onOpenFile={openFile}
        onNewRequest={() => addQuickTab()}
        onSelectFile={openFileInTab}
        onRefresh={() => {
          const w = workspace();
          if (w) loadWorkspace(w.path);
        }}
      />

      <main class="flex min-w-0 flex-1 flex-col">
        <header class="flex items-center gap-2 border-b border-border bg-surface px-3 py-2">
          <button
            onClick={() => addQuickTab()}
            class="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white shadow hover:bg-accent-2 focus:outline-none focus:ring-2 focus:ring-accent/50 transition"
          >
            <Plus size={16} />
            New request
          </button>
          <button
            onClick={openFile}
            class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface-2 px-3 py-1.5 text-sm text-text-2 hover:bg-white/5 focus:outline-none focus:ring-2 focus:ring-accent/50 transition"
          >
            <FileText size={16} />
            Open file
          </button>
          <button
            onClick={openFolder}
            class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface-2 px-3 py-1.5 text-sm text-text-2 hover:bg-white/5 focus:outline-none focus:ring-2 focus:ring-accent/50 transition"
          >
            <FolderOpen size={16} />
            Open folder
          </button>
          <div class="ml-auto text-xs text-text-2">{version()}</div>
        </header>

        <Show
          when={tabs.length}
          fallback={
            <div class="flex flex-1 items-center justify-center text-text-2">
              Open or create a request to get started
            </div>
          }
        >
          <div class="flex border-b border-border bg-bg">
            <For each={tabs}>
              {(tab) => (
                <div
                  onClick={() => setActiveTabId(tabId(tab))}
                  class={`group flex cursor-pointer items-center gap-2 border-r border-border px-3 py-2 text-sm transition ${
                    tabId(tab) === activeTabId()
                      ? "bg-surface text-text"
                      : "text-text-2 hover:bg-surface/50"
                  }`}
                >
                  <span class="max-w-[160px] truncate">{tabName(tab)}</span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      closeTab(tabId(tab));
                    }}
                    class="rounded p-0.5 text-text-2 opacity-0 transition hover:bg-white/10 hover:text-text group-hover:opacity-100"
                  >
                    <X size={12} />
                  </button>
                </div>
              )}
            </For>
          </div>

          <div class="grid flex-1 grid-rows-2 overflow-hidden">
            <Show when={activeTab()}>
              <RequestPanel
                kind={activeTab()!.kind}
                quick={
                  activeTab()!.kind === "quick" ? activeTab()!.req : undefined
                }
                file={
                  activeTab()!.kind === "file" ? activeTab()!.file : undefined
                }
                onQuickChange={(req) => updateQuickTab(req.id, req)}
                onFileChange={(file) => updateFileTab(file.path, file)}
                onRun={runActiveTab}
                onSave={saveActiveFile}
                loading={loading()}
              />
            </Show>
            <ResponsePanel result={result()} />
          </div>
        </Show>
      </main>
    </div>
  );
}
