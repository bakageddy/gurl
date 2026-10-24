import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { runs } from "./stores/runs.svelte";
import { settings } from "./stores/settings.svelte";
import { tabs } from "./stores/tabs.svelte";
import { ui } from "./stores/ui.svelte";
import { workspace } from "./stores/workspace.svelte";
import { basename, dirname } from "./format";

const SCENARIO_TEMPLATE = "GET https://example.org\n\nHTTP 200\n";

export async function openWorkspaceDialog() {
  const picked = await openDialog({ directory: true, multiple: false, title: "Open workspace folder" });
  if (typeof picked === "string") await openWorkspacePath(picked);
}

export async function openWorkspacePath(path: string) {
  try {
    await workspace.open(path);
    settings.lastWorkspace = workspace.root;
    settings.save();
  } catch (e) {
    ui.error(e);
  }
}

export async function openFileDialog() {
  const picked = await openDialog({
    multiple: false,
    filters: [{ name: "HURL", extensions: ["hurl", "hur"] }],
  });
  if (typeof picked === "string") await tabs.openFile(picked);
}

export function depthOf(path: string): number {
  if (!workspace.root) return 0;
  return path.slice(workspace.root.length).split(/[\\/]/).filter(Boolean).length;
}

export async function newScenario(dir: string | null = workspace.targetDir()) {
  if (!dir) return;
  const name = await ui.ask({ title: "New scenario", label: "File name", value: "scenario.hurl", confirmLabel: "Create" });
  if (!name) return;
  try {
    const path = await workspace.newFile(dir, name.trim(), SCENARIO_TEMPLATE);
    await tabs.openFile(path);
  } catch (e) {
    ui.error(e);
  }
}

export async function newFolder(parent: string | null = workspace.targetDir()) {
  if (!parent) return;
  if (depthOf(parent) >= workspace.maxDepth) {
    ui.toast(`Maximum nesting depth of ${workspace.maxDepth} reached`, "error");
    return;
  }
  const name = await ui.ask({ title: "New folder", label: "Folder name", confirmLabel: "Create" });
  if (!name) return;
  try {
    await workspace.newDir(parent, name.trim());
  } catch (e) {
    ui.error(e);
  }
}

export async function renameEntry(path: string) {
  const name = await ui.ask({ title: "Rename", label: "New name", value: basename(path), confirmLabel: "Rename" });
  if (!name || name === basename(path)) return;
  try {
    const to = await workspace.rename(path, name.trim());
    tabs.rewritePaths(path, to);
  } catch (e) {
    ui.error(e);
  }
}

export async function deleteEntry(path: string) {
  const ok = await ui.confirm({
    title: `Delete ${basename(path)}?`,
    message: "This removes it from disk and cannot be undone.",
    confirmLabel: "Delete",
    danger: true,
  });
  if (!ok) return;
  try {
    await workspace.remove(path);
    tabs.rewritePaths(path, null);
  } catch (e) {
    ui.error(e);
  }
}

export async function duplicateEntry(path: string) {
  try {
    const to = await workspace.duplicate(path);
    await tabs.openFile(to);
  } catch (e) {
    ui.error(e);
  }
}

export async function runDirectory(dir: string, recursive = settings.recursive) {
  const label = `${basename(dir)}${recursive ? " (recursive)" : ""}`;
  const state = await runs.start({ kind: "dir", path: dir, recursive }, label);
  tabs.openReport(state, `Run · ${label}`);
}

export async function runFilePath(path: string) {
  const tab = await tabs.openFile(path);
  if (tab && tab.kind === "doc") await tab.execute("file");
}

export async function runActive() {
  const t = tabs.active;
  if (t?.kind === "doc") await t.execute(t.path ? "file" : "text");
}

export async function saveActive() {
  const t = tabs.active;
  if (t?.kind === "doc") {
    const ok = await t.save();
    if (ok) {
      const d = dirname(t.path!);
      workspace.refresh([d]);
    }
  }
}
