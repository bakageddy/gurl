import { createMemo, createSignal, For, Show } from "solid-js";
import type { HurlCall, HurlEntry, RunResult } from "../types";
import { AlertCircle, CheckCircle, Clock, FileJson, List, Shield } from "lucide-solid";

function syntaxHighlight(json: string): string {
  return json
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(
      /("(?:[^"\\]|\\.)*")(\s*:\s*)?/g,
      (_, str, colon) => {
        if (colon) return `<span class="json-key">${str}</span>${colon}`;
        return `<span class="json-string">${str}</span>`;
      }
    )
    .replace(/\b(true|false)\b/g, '<span class="json-boolean">$1</span>')
    .replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="json-number">$1</span>');
}

interface Props {
  result: RunResult | null;
}

const TAB_BUTTON =
  "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition focus:outline-none";
const TAB_ACTIVE = "bg-surface-2 text-text";
const TAB_INACTIVE = "text-text-2 hover:bg-white/5";

export function ResponsePanel(props: Props) {
  const [activeTab, setActiveTab] = createSignal("body");
  const [activeEntry, setActiveEntry] = createSignal(0);

  const report = createMemo(() => props.result?.report);

  const entries = createMemo<HurlEntry[]>(() => report()?.entries || []);

  const currentEntry = createMemo<HurlEntry | undefined>(
    () => entries()[activeEntry()]
  );

  const currentCall = createMemo<HurlCall | undefined>(
    () => currentEntry()?.calls[0]
  );

  const responseBody = createMemo<string>(() => {
    return currentCall()?.response?.body || "";
  });

  const prettyBody = createMemo<string>(() => {
    const raw = responseBody();
    if (!raw) return "";
    try {
      return syntaxHighlight(JSON.stringify(JSON.parse(raw), null, 2));
    } catch {
      return raw
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
    }
  });

  const status = createMemo(() => currentCall()?.response?.status ?? 0);
  const duration = createMemo(
    () => currentEntry()?.time ?? props.result?.duration_ms ?? 0
  );

  return (
    <div class="flex flex-col overflow-hidden bg-bg">
      <Show
        when={props.result}
        fallback={
          <div class="flex flex-1 flex-col items-center justify-center gap-3 text-text-2">
            <FileJson size={40} class="opacity-30" />
            <p class="text-sm">Run a request to see the response</p>
          </div>
        }
      >
        <div class="flex flex-wrap items-center gap-2 border-b border-border bg-surface px-3 py-2">
          {props.result!.ok ? (
            <span class="inline-flex items-center gap-1 rounded-full bg-success/15 px-2.5 py-1 text-xs font-medium text-success">
              <CheckCircle size={12} />
              Success
            </span>
          ) : (
            <span class="inline-flex items-center gap-1 rounded-full bg-danger/15 px-2.5 py-1 text-xs font-medium text-danger">
              <AlertCircle size={12} />
              Failed
            </span>
          )}
          <Show when={status()}>
            <span class="rounded-full bg-surface-2 px-2.5 py-1 text-xs text-text">
              HTTP {status()}
            </span>
          </Show>
          <span class="inline-flex items-center gap-1 rounded-full bg-surface-2 px-2.5 py-1 text-xs text-text">
            <Clock size={12} />
            {duration()} ms
          </span>
          <Show when={entries().length > 1}>
            <select
              value={activeEntry()}
              onChange={(e) => setActiveEntry(Number(e.currentTarget.value))}
              class="ml-auto rounded-lg border border-border bg-surface-2 px-2 py-1 text-xs text-text outline-none"
            >
              <For each={entries()}>
                {(entry, i) => (
                  <option value={i()}>
                    #{entry.index} {entry.calls[0]?.request?.method} {entry.calls[0]?.request?.url}
                  </option>
                )}
              </For>
            </select>
          </Show>
        </div>

        <Show when={props.result!.error}>
          <div class="mx-3 mt-2 rounded-lg border border-danger/30 bg-danger/10 p-3 font-mono text-xs text-danger">
            {props.result!.error}
          </div>
        </Show>

        <div class="flex items-center gap-1 border-b border-border bg-surface px-2 py-1">
          {[
            { id: "body", label: "Body" },
            { id: "headers", label: "Headers" },
            { id: "cookies", label: "Cookies" },
            { id: "asserts", label: "Asserts" },
            { id: "captures", label: "Captures" },
            { id: "certificate", label: "Certificate" },
            { id: "raw", label: "JSON report" },
          ].map((tab) => (
            <button
              onClick={() => setActiveTab(tab.id)}
              class={`${TAB_BUTTON} ${
                tab.id === activeTab() ? TAB_ACTIVE : TAB_INACTIVE
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div class="flex-1 overflow-auto p-4">
          {activeTab() === "body" && (
            <div
              class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed text-text"
              innerHTML={prettyBody()}
            />
          )}
          {activeTab() === "headers" && (
            <div class="space-y-2">
              <For each={currentCall()?.response?.headers || []}>
                {(h) => (
                  <div class="rounded-lg border border-border bg-surface p-2">
                    <div class="text-xs font-medium text-text-2">{h.name}</div>
                    <div class="font-mono text-sm text-text">{h.value}</div>
                  </div>
                )}
              </For>
            </div>
          )}
          {activeTab() === "cookies" && (
            <div class="space-y-2">
              <For each={currentCall()?.response?.cookies || []}>
                {(c) => (
                  <div class="rounded-lg border border-border bg-surface p-2">
                    <div class="flex items-center gap-2">
                      <span class="font-mono text-sm font-medium text-text">{c.name}</span>
                      <span class="font-mono text-sm text-text-2">=</span>
                      <span class="font-mono text-sm text-warning">{c.value}</span>
                    </div>
                    <Show when={c.expires}>
                      <div class="mt-1 text-xs text-text-2">expires {c.expires}</div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          )}
          {activeTab() === "asserts" && (
            <div class="space-y-2">
              <For each={currentEntry()?.asserts || []}>
                {(a) => (
                  <div
                    class={`rounded-lg border p-2 text-sm ${
                      a.success
                        ? "border-success/30 bg-success/10 text-success"
                        : "border-danger/30 bg-danger/10 text-danger"
                    }`}
                  >
                    <div class="flex items-center gap-2 font-medium">
                      {a.success ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
                      line {a.line}: {a.success ? "passed" : "failed"}
                    </div>
                    <Show when={a.message}>
                      <pre class="mt-1 whitespace-pre-wrap font-mono text-xs opacity-90">
                        {a.message}
                      </pre>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          )}
          {activeTab() === "captures" && (
            <div class="space-y-2">
              <For each={currentEntry()?.captures || []}>
                {(c) => (
                  <div class="rounded-lg border border-border bg-surface p-2">
                    <span class="text-sm font-medium text-text">{c.name}</span>
                    <span class="mx-2 text-text-2">=</span>
                    <span class="font-mono text-sm text-accent">
                      {JSON.stringify(c.value)}
                    </span>
                  </div>
                )}
              </For>
            </div>
          )}
          {activeTab() === "certificate" && (
            <div class="space-y-2">
              <Show
                when={currentCall()?.response?.certificate}
                fallback={
                  <div class="text-sm text-text-2">No TLS certificate available</div>
                }
              >
                <For
                  each={Object.entries(currentCall()!.response.certificate!).filter(
                    ([_, v]) => v != null
                  )}
                >
                  {([k, v]) => (
                    <div class="rounded-lg border border-border bg-surface p-2">
                      <div class="text-xs font-medium text-text-2">{k}</div>
                      <div class="break-words font-mono text-xs text-text">{String(v)}</div>
                    </div>
                  )}
                </For>
              </Show>
            </div>
          )}
          {activeTab() === "raw" && (
            <pre class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-text-2">
              {JSON.stringify(props.result!.report, null, 2)}
            </pre>
          )}
        </div>
      </Show>
    </div>
  );
}
