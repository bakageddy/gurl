import { createSignal, Show, type JSX } from "solid-js";
import { Play, Save, Loader2, FileCode } from "lucide-solid";
import type { FileRequest, QuickRequest } from "../types";
import { KeyValueEditor } from "./KeyValueEditor";
import { OptionsPanel } from "./OptionsPanel";

interface Props {
  kind: "quick" | "file";
  quick?: QuickRequest;
  file?: FileRequest;
  onQuickChange?: (req: QuickRequest) => void;
  onFileChange?: (file: FileRequest) => void;
  onRun: () => void;
  onSave?: () => void;
  loading?: boolean;
}

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "CUSTOM"];
const TAB_BUTTON =
  "px-3 py-1.5 text-xs font-medium rounded-md transition focus:outline-none";
const TAB_ACTIVE = "bg-accent text-white shadow-sm";
const TAB_INACTIVE = "text-text-2 hover:bg-white/5";

export function RequestPanel(props: Props) {
  const [activeTab, setActiveTab] = createSignal("params");

  if (props.kind === "file" && props.file && props.onFileChange) {
    return (
      <div class="flex flex-col overflow-hidden border-b border-border bg-bg">
        <div class="flex items-center gap-2 border-b border-border bg-surface px-3 py-2">
          <FileCode size={16} class="text-accent" />
          <input
            type="text"
            value={props.file.name}
            onInput={(e) =>
              props.onFileChange!({ ...props.file!, name: e.currentTarget.value })
            }
            class="min-w-0 flex-1 bg-transparent text-sm font-medium text-text outline-none"
          />
          <Show when={props.file.dirty}>
            <span class="text-xs text-warning">● unsaved</span>
          </Show>
          <button
            onClick={props.onSave}
            disabled={!props.file.dirty || props.loading}
            class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface-2 px-3 py-1.5 text-xs font-medium text-text hover:bg-white/5 disabled:opacity-40 transition"
          >
            <Save size={14} />
            Save
          </button>
          <button
            onClick={props.onRun}
            disabled={props.loading}
            class="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-2 disabled:opacity-60 transition"
          >
            {props.loading ? <Loader2 size={14} class="animate-spin" /> : <Play size={14} />}
            {props.loading ? "Running…" : "Send"}
          </button>
        </div>
        <textarea
          value={props.file.content}
          onInput={(e) =>
            props.onFileChange!({
              ...props.file!,
              content: e.currentTarget.value,
              dirty: true,
            })
          }
          class="flex-1 resize-none bg-[#0d0f14] p-4 font-mono text-sm leading-relaxed text-text outline-none"
          spellcheck={false}
        />
      </div>
    );
  }

  const req = () => props.quick!;
  const update = (patch: Partial<QuickRequest>) => {
    props.onQuickChange?.({ ...req(), ...patch });
  };

  function buildUrl(): string {
    const base = req().url || "";
    const pairs = req()
      .params.filter((p) => p.enabled && p.name)
      .map((p) => `${encodeURIComponent(p.name)}=${encodeURIComponent(p.value)}`);
    if (!pairs.length) return base;
    const sep = base.includes("?") ? "&" : "?";
    return `${base}${sep}${pairs.join("&")}`;
  }

  function generateHurlText(): string {
    if (req().rawMode) return req().body;
    const method = req().method === "CUSTOM" ? "GET" : req().method;
    const url = buildUrl();
    const headerLines = req()
      .headers.filter((h) => h.enabled && h.name)
      .map((h) => `${h.name}: ${h.value}`);
    const parts = [`${method} ${url}`, ...headerLines];
    if (req().body) {
      parts.push("");
      parts.push(req().body);
    }
    return parts.join("\n");
  }

  const tabs: { id: string; label: string; render: () => JSX.Element }[] = [
    {
      id: "params",
      label: `Params ${req().params.length ? `(${req().params.length})` : ""}`,
      render: () => (
        <KeyValueEditor
          items={req().params}
          onChange={(params) => update({ params })}
        />
      ),
    },
    {
      id: "headers",
      label: `Headers ${req().headers.length ? `(${req().headers.length})` : ""}`,
      render: () => (
        <KeyValueEditor
          items={req().headers}
          onChange={(headers) => update({ headers })}
        />
      ),
    },
    {
      id: "body",
      label: "Body",
      render: () => (
        <textarea
          value={req().body}
          onInput={(e) => update({ body: e.currentTarget.value })}
          placeholder="Request body (JSON, text, etc.)"
          class="h-full w-full resize-none rounded-lg border border-border bg-[#0d0f14] p-3 font-mono text-sm leading-relaxed text-text outline-none transition placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
          spellcheck={false}
        />
      ),
    },
    {
      id: "raw",
      label: "Raw Hurl",
      render: () => (
        <textarea
          value={req().rawMode ? req().body : generateHurlText()}
          onInput={(e) => update({ body: e.currentTarget.value, rawMode: true })}
          placeholder="Write any Hurl file content here"
          class="h-full w-full resize-none rounded-lg border border-border bg-[#0d0f14] p-3 font-mono text-sm leading-relaxed text-text outline-none transition placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
          spellcheck={false}
        />
      ),
    },
    {
      id: "options",
      label: "Options",
      render: () => (
        <OptionsPanel
          options={req().options}
          onChange={(options) => update({ options })}
        />
      ),
    },
  ];

  const currentTab = () => tabs.find((t) => t.id === activeTab()) || tabs[0];

  return (
    <div class="flex flex-col overflow-hidden border-b border-border bg-bg">
      <div class="flex items-center gap-2 bg-surface px-3 py-2">
        <select
          value={req().method}
          onChange={(e) => update({ method: e.currentTarget.value })}
          class="rounded-lg border border-border bg-surface-2 px-2 py-1.5 text-sm font-semibold text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
        >
          {METHODS.map((m) => (
            <option value={m}>{m}</option>
          ))}
        </select>
        <input
          type="text"
          value={req().url}
          onInput={(e) => update({ url: e.currentTarget.value })}
          placeholder="https://api.example.com/resource"
          class="min-w-0 flex-1 rounded-lg border border-border bg-surface-2 px-3 py-1.5 text-sm text-text outline-none transition placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
        />
        <button
          onClick={props.onRun}
          disabled={props.loading}
          class="inline-flex shrink-0 items-center gap-1.5 rounded-lg bg-accent px-4 py-1.5 text-sm font-medium text-white shadow hover:bg-accent-2 disabled:opacity-60 transition"
        >
          {props.loading ? (
            <Loader2 size={16} class="animate-spin" />
          ) : (
            <Play size={16} />
          )}
          {props.loading ? "Running…" : "Send"}
        </button>
      </div>

      <div class="flex items-center gap-1 border-b border-border bg-surface px-2 py-1">
        {tabs.map((tab) => (
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

      <div class="flex-1 overflow-hidden p-3">{currentTab().render()}</div>
    </div>
  );
}
