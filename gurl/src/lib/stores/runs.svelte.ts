import { cancelRun, listRuns, loadRun, runScope, type ScopeRequest } from "../api";
import type { FileRun, RunEvent, RunSummary } from "../types";
import { settings } from "./settings.svelte";
import { ui } from "./ui.svelte";

export type FileState = "pending" | "running" | "done";

/** Live (or loaded) state of one run, shared with whichever views display it. */
export class RunState {
  id = $state("");
  label: string;
  files = $state<string[]>([]);
  states = $state<FileState[]>([]);
  results = $state<Array<FileRun | undefined>>([]);
  summary = $state<RunSummary | null>(null);
  error = $state<string | null>(null);
  running = $state(true);

  constructor(label: string) {
    this.label = label;
  }

  handle(e: RunEvent) {
    switch (e.event) {
      case "started":
        this.id = e.runId;
        this.files = e.files;
        this.states = e.files.map(() => "pending");
        this.results = e.files.map(() => undefined);
        break;
      case "fileStarted":
        this.states[e.index] = "running";
        break;
      case "fileFinished":
        this.states[e.index] = "done";
        this.results[e.index] = e.run;
        break;
      case "done":
        this.summary = e.summary;
        this.running = false;
        break;
    }
  }

  get passed() {
    return this.results.filter((r) => r?.ok).length;
  }
  get failed() {
    return this.results.filter((r) => r && !r.ok).length;
  }
}

class Runs {
  history = $state<RunSummary[]>([]);
  /** Most recent run, whatever started it. */
  latest = $state<RunState | null>(null);

  async start(request: Omit<ScopeRequest, "options" | "jobs">, label: string): Promise<RunState> {
    const state = new RunState(label);
    this.latest = state;
    runScope(
      { ...request, jobs: settings.jobs, options: $state.snapshot(settings.runOptions) },
      (e) => state.handle(e),
    )
      .then(() => this.refreshHistory())
      .catch((e) => {
        state.error = typeof e === "string" ? e : String(e);
        state.running = false;
        ui.error(e);
      });
    return state;
  }

  cancel(state: RunState) {
    if (state.id) cancelRun(state.id).catch(() => {});
  }

  async refreshHistory() {
    try {
      this.history = await listRuns();
    } catch {
      /* ignore */
    }
  }

  async open(summary: RunSummary): Promise<RunState> {
    const results = await loadRun(summary.id);
    const state = new RunState(summary.label);
    state.id = summary.id;
    state.files = results.map((r) => r.path);
    state.states = results.map(() => "done");
    state.results = results;
    state.summary = summary;
    state.running = false;
    return state;
  }
}

export const runs = new Runs();
