import { createSignal, For, Show } from "solid-js";
import { ChevronRight, ChevronDown, Folder, FileCode, RefreshCw, FolderOpen, FileText, Plus } from "lucide-solid";
import type { TreeEntry, Workspace } from "../types";

interface Props {
  workspace: Workspace | null;
  activePath?: string;
  onOpenFolder: () => void;
  onOpenFile: () => void;
  onNewRequest: () => void;
  onSelectFile: (path: string) => void;
  onRefresh: () => void;
}

function TreeNode(props: {
  entry: TreeEntry;
  depth: number;
  activePath?: string;
  onSelectFile: (path: string) => void;
}) {
  const [open, setOpen] = createSignal(true);
  const isActive = () => props.activePath === props.entry.path;

  return (
    <li>
      <button
        onClick={() =>
          props.entry.is_dir
            ? setOpen(!open())
            : props.onSelectFile(props.entry.path)
        }
        class={`flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-left text-sm transition ${
          isActive()
            ? "bg-accent/20 text-accent"
            : "text-text-2 hover:bg-white/5 hover:text-text"
        }`}
        style={{ "padding-left": `${8 + props.depth * 14}px` }}
      >
        <span class="flex h-4 w-4 items-center justify-center text-text-2">
          {props.entry.is_dir ? (
            open() ? <ChevronDown size={14} /> : <ChevronRight size={14} />
          ) : (
            <FileCode size={14} />
          )}
        </span>
        <span class="truncate">{props.entry.name}</span>
      </button>
      <Show when={props.entry.is_dir && open()}>
        <ul class="mt-0.5">
          <For each={props.entry.children}>
            {(child) => (
              <TreeNode
                entry={child}
                depth={props.depth + 1}
                activePath={props.activePath}
                onSelectFile={props.onSelectFile}
              />
            )}
          </For>
        </ul>
      </Show>
    </li>
  );
}

export function Sidebar(props: Props) {
  return (
    <aside class="flex w-64 flex-col border-r border-border bg-surface">
      <div class="flex items-center gap-2 border-b border-border px-3 py-2 font-semibold text-text">
        <Folder size={16} class="text-accent" />
        Workspace
      </div>
      <div class="flex gap-2 border-b border-border px-3 py-2">
        <button
          onClick={props.onNewRequest}
          class="inline-flex flex-1 items-center justify-center gap-1 rounded-lg bg-accent px-2 py-1.5 text-xs font-medium text-white hover:bg-accent-2 transition"
        >
          <Plus size={14} />
          New
        </button>
        <button
          onClick={props.onOpenFile}
          title="Open file"
          class="rounded-lg border border-border bg-surface-2 p-1.5 text-text-2 hover:bg-white/5 transition"
        >
          <FileText size={16} />
        </button>
        <button
          onClick={props.onOpenFolder}
          title="Open folder"
          class="rounded-lg border border-border bg-surface-2 p-1.5 text-text-2 hover:bg-white/5 transition"
        >
          <FolderOpen size={16} />
        </button>
        <button
          onClick={props.onRefresh}
          title="Refresh"
          class="rounded-lg border border-border bg-surface-2 p-1.5 text-text-2 hover:bg-white/5 transition"
        >
          <RefreshCw size={16} />
        </button>
      </div>
      <div class="flex-1 overflow-auto px-2 py-3">
        <Show
          when={props.workspace}
          fallback={
            <div class="px-2 py-8 text-center text-xs leading-relaxed text-text-2">
              Open a folder that contains
              <br />
              .hurl files to start browsing.
            </div>
          }
        >
          <div class="mb-2 break-all px-2 text-xs text-text-2">
            {props.workspace!.path}
          </div>
          <ul>
            <For each={props.workspace!.tree}>
              {(entry) => (
                <TreeNode
                  entry={entry}
                  depth={0}
                  activePath={props.activePath}
                  onSelectFile={props.onSelectFile}
                />
              )}
            </For>
          </ul>
        </Show>
      </div>
    </aside>
  );
}
