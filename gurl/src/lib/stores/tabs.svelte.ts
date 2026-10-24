import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { addResponseItems, emitHurl, parseHurl, readTextFile, writeTextFile } from "../api";
import type { AssertItems } from "../assert-target";
import type { Diagnostic, HurlDoc } from "../types";
import { basename } from "../format";
import type { RunState } from "./runs.svelte";
import { runs } from "./runs.svelte";
import { workspace } from "./workspace.svelte";
import { ui } from "./ui.svelte";

export type DocMode = "builder" | "text";

const UNTITLED_TEXT = "GET https://example.org\n\nHTTP 200\n";
let untitledSeq = 0;

export class DocTab {
  readonly kind = "doc" as const;
  id: string;
  path = $state<string | null>(null);
  name = $state("");
  text = $state("");
  saved = $state("");
  model = $state<HurlDoc | null>(null);
  diagnostics = $state<Diagnostic[]>([]);
  lossless = $state(true);
  droppedComments = $state(0);
  mode = $state<DocMode>("builder");
  run = $state<RunState | null>(null);
  dirty = $derived(this.text !== this.saved);

  private parseTimer: ReturnType<typeof setTimeout> | undefined;
  private emitTimer: ReturnType<typeof setTimeout> | undefined;
  private seq = 0;

  constructor(path: string | null, text: string) {
    this.path = path;
    this.id = path ?? `untitled-${++untitledSeq}`;
    this.name = path ? basename(path) : `Untitled ${untitledSeq}.hurl`;
    this.text = text;
    this.saved = path ? text : "";
    void this.reparse();
  }

  get title() {
    return this.name;
  }

  /** Text view edited the source. */
  setText(text: string) {
    this.text = text;
    clearTimeout(this.emitTimer);
    clearTimeout(this.parseTimer);
    this.parseTimer = setTimeout(() => void this.reparse(), 200);
  }

  async reparse() {
    const seq = ++this.seq;
    try {
      const out = await parseHurl(this.text);
      if (seq !== this.seq) return;
      this.diagnostics = out.diagnostics;
      this.lossless = out.lossless;
      this.droppedComments = out.droppedComments;
      if (out.model) this.model = out.model;
      else if (!this.model) this.model = null;
      if (out.diagnostics.length) this.modelValid = false;
      else this.modelValid = true;
    } catch (e) {
      ui.error(e);
    }
  }
  modelValid = $state(true);

  /** Builder mutated `model`; regenerate the text (debounced). */
  touch() {
    clearTimeout(this.emitTimer);
    this.emitTimer = setTimeout(() => void this.flushModel(), 120);
  }

  async flushModel() {
    clearTimeout(this.emitTimer);
    if (!this.model) return;
    const seq = ++this.seq;
    try {
      const text = await emitHurl($state.snapshot(this.model) as HurlDoc);
      if (seq !== this.seq) return;
      this.text = text;
      this.diagnostics = [];
      this.modelValid = true;
      this.lossless = true;
    } catch (e) {
      ui.error(e);
    }
  }

  /** Make sure model and text agree before switching views / saving / running. */
  async sync() {
    if (this.emitTimer) await this.flushModel();
    if (this.parseTimer) {
      clearTimeout(this.parseTimer);
      this.parseTimer = undefined;
      await this.reparse();
    }
  }

  async save(): Promise<boolean> {
    await this.sync();
    let path = this.path;
    if (!path) {
      const picked = await saveDialog({
        defaultPath: workspace.root ?? undefined,
        filters: [{ name: "HURL", extensions: ["hurl"] }],
      });
      if (!picked) return false;
      path = /\.(hurl|hur)$/.test(picked) ? picked : `${picked}.hurl`;
    }
    try {
      await writeTextFile(path, this.text);
    } catch (e) {
      ui.error(e);
      return false;
    }
    this.path = path;
    this.id = path;
    this.name = basename(path);
    this.saved = this.text;
    return true;
  }

  /** Reload from disk (file changed externally and there are no local edits). */
  async reloadFromDisk() {
    if (!this.path || this.dirty) return;
    try {
      const text = await readTextFile(this.path);
      if (text !== this.text) {
        this.text = text;
        this.saved = text;
        await this.reparse();
      }
    } catch {
      /* deleted */
    }
  }

  async execute(kind: "file" | "text") {
    await this.sync();
    this.run = await runs.start(
      kind === "text" || !this.path
        ? { kind: "text", content: this.text, path: this.path ?? undefined }
        : { kind: "file", path: this.path, content: this.dirty ? this.text : undefined },
      this.name,
    );
    this.run.sourceText = this.text;
  }

  /** Line the text view should scroll to after an edit made outside the editor. */
  pendingReveal = $state<number | null>(null);

  /** Adds asserts / captures to entry `entryIndex` (1-based) by splicing the text, so comments
   *  and formatting elsewhere survive in both views. */
  async addResponseItems(entryIndex: number, items: AssertItems) {
    await this.sync();
    const changed = this.run?.sourceText != null && this.run.sourceText !== this.text;
    const edit = await addResponseItems(this.text, entryIndex, items.asserts ?? [], items.captures ?? []);
    this.setText(edit.text);
    this.pendingReveal = edit.firstLine;
    const what = items.captures?.length ? "capture" : "assert";
    ui.toast(`Added ${edit.added} ${what}${edit.added === 1 ? "" : "s"} · line ${edit.firstLine}`);
    if (changed) ui.toast("The file changed since this run: check that it landed on the right request");
  }
}

export class VarsTab {
  readonly kind = "vars" as const;
  id: string;
  constructor(public dir: string) {
    this.id = `vars:${dir}`;
  }
  get title() {
    return `${basename(this.dir)} · variables`;
  }
}

export class ReportTab {
  readonly kind = "report" as const;
  id: string;
  constructor(public run: RunState, public title: string) {
    this.id = `report:${run.id || Math.random().toString(36).slice(2)}`;
  }
}

export type Tab = DocTab | VarsTab | ReportTab;

class Tabs {
  list = $state<Tab[]>([]);
  activeId = $state<string | null>(null);
  active = $derived(this.list.find((t) => t.id === this.activeId) ?? null);

  private add(tab: Tab) {
    this.list.push(tab);
    this.activeId = tab.id;
    return tab;
  }

  newUntitled() {
    return this.add(new DocTab(null, UNTITLED_TEXT));
  }

  async openFile(path: string) {
    const existing = this.list.find((t) => t.id === path);
    if (existing) {
      this.activeId = existing.id;
      return existing;
    }
    try {
      const text = await readTextFile(path);
      return this.add(new DocTab(path, text));
    } catch (e) {
      ui.error(e);
    }
  }

  openVars(dir: string) {
    const id = `vars:${dir}`;
    const existing = this.list.find((t) => t.id === id);
    if (existing) {
      this.activeId = id;
      return existing;
    }
    return this.add(new VarsTab(dir));
  }

  openReport(run: RunState, title: string) {
    const existing = run.id ? this.list.find((t) => t.id === `report:${run.id}`) : undefined;
    if (existing) {
      this.activeId = existing.id;
      return existing;
    }
    return this.add(new ReportTab(run, title));
  }

  async close(id: string) {
    const i = this.list.findIndex((t) => t.id === id);
    if (i < 0) return;
    const tab = this.list[i];
    if (tab.kind === "doc" && tab.dirty) {
      const ok = await ui.confirm({
        title: `Close ${tab.name}?`,
        message: "This file has unsaved changes that will be lost.",
        confirmLabel: "Discard changes",
        danger: true,
      });
      if (!ok) return;
    }
    this.list.splice(i, 1);
    if (this.activeId === id) this.activeId = this.list[Math.min(i, this.list.length - 1)]?.id ?? null;
  }

  /** Called when a file/dir is renamed or removed on disk by the app. */
  rewritePaths(from: string, to: string | null) {
    for (const t of [...this.list]) {
      if (t.kind === "doc" && t.path && (t.path === from || t.path.startsWith(from + "/") || t.path.startsWith(from + "\\"))) {
        if (to === null) {
          t.saved = "\u0000"; // keep content, mark dirty
          t.path = null;
          t.id = `untitled-${++untitledSeq}`;
          if (this.activeId === from) this.activeId = t.id;
        } else {
          const np = to + t.path.slice(from.length);
          if (this.activeId === t.id) this.activeId = np;
          t.path = np;
          t.id = np;
          t.name = basename(np);
        }
      }
    }
  }
}

export const tabs = new Tabs();
