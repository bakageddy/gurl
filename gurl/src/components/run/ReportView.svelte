<script lang="ts">
  import { setContext } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import Icon from "../Icon.svelte";
  import FileRunView from "./FileRunView.svelte";
  import { exportReport } from "../../lib/api";
  import { basename, ms, relative, timeAgo } from "../../lib/format";
  import { runs, type RunState } from "../../lib/stores/runs.svelte";
  import { tabs } from "../../lib/stores/tabs.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { workspace } from "../../lib/stores/workspace.svelte";
  import type { ReportFormat } from "../../lib/types";
  import { ASSERT_TARGET, type AssertTarget } from "../../lib/assert-target";

  let { run }: { run: RunState } = $props();

  let selected = $state(0);

  // "Add assert" from a report opens the file and edits it there. Text runs have no file.
  const isFile = (p: string | undefined) => !!p && (p.includes("/") || p.includes("\\"));
  setContext<AssertTarget>(ASSERT_TARGET, {
    enabled: () => !run.running && isFile(run.files[selected]),
    reason: () => (run.running ? "Wait for the run to finish" : "Save the scenario first, then run it from the file"),
    add: async (entryIndex, items) => {
      const tab = await tabs.openFile(run.files[selected]);
      if (tab?.kind === "doc") await tab.addResponseItems(entryIndex, items);
    },
  });
  const total = $derived(run.files.length);
  const done = $derived(run.states.filter((s) => s === "done").length);
  const maxDur = $derived(Math.max(1, ...run.results.map((r) => r?.durationMs ?? 0)));
  const dur = $derived(run.summary?.durationMs);

  function shortName(p: string) {
    return workspace.root ? relative(workspace.root, p) : basename(p);
  }

  async function doExport(format: ReportFormat) {
    try {
      let dir: string | undefined;
      let openAfter = format === "html";
      if (!openAfter) {
        const picked = await openDialog({ directory: true, multiple: false, title: `Choose a folder for the ${format.toUpperCase()} report` });
        if (typeof picked !== "string") return;
        dir = picked;
      }
      const file = await exportReport(run.id, format, openAfter, dir);
      ui.toast(openAfter ? "Opened HURL's HTML report" : `Saved ${file}`);
    } catch (e) {
      ui.error(e);
    }
  }

  function openFile(path: string) {
    void tabs.openFile(path);
  }
</script>

<div class="report">
  <header>
    <div class="title">
      <h2>{run.label}</h2>
      <div class="stats">
        {#if run.running}
          <span class="badge warn">Running {done}/{total}</span>
        {:else if run.summary?.cancelled}
          <span class="badge warn">Cancelled</span>
        {/if}
        <span class="badge ok">{run.passed} passed</span>
        <span class="badge" class:fail={run.failed > 0}>{run.failed} failed</span>
        {#if dur !== undefined}<span class="muted">{ms(dur)}</span>{/if}
        {#if run.summary}<span class="subtle">{timeAgo(run.summary.startedAt)}</span>{/if}
      </div>
    </div>
    <div class="actions">
      {#if run.running}
        <button class="btn danger" onclick={() => runs.cancel(run)}><Icon name="stop" size={12} /> Cancel</button>
      {:else}
        <button class="btn" onclick={() => doExport("html")} title="Generate HURL's own HTML report and open it"><Icon name="external" size={13} /> HURL HTML report</button>
        <select class="field export" onchange={(e) => { const v = e.currentTarget.value as ReportFormat; e.currentTarget.value = ""; if (v) void doExport(v); }}>
          <option value="">Export…</option>
          <option value="junit">JUnit XML</option>
          <option value="tap">TAP</option>
          <option value="json">JSON</option>
          <option value="html">HTML (choose again)</option>
        </select>
      {/if}
    </div>
  </header>
  {#if run.running}
    <div class="progress"><div style:width="{total ? (done / total) * 100 : 0}%"></div></div>
  {/if}
  {#if run.error}<div class="banner error m">{run.error}</div>{/if}

  <div class="split">
    <ul class="files">
      {#each run.files as f, i (i)}
        {@const r = run.results[i]}
        <li>
          <button class="file" class:sel={selected === i} onclick={() => (selected = i)} ondblclick={() => openFile(f)} title="{f} — double-click to open">
            <span class="st">
              {#if run.states[i] === "running"}<span class="spin"></span>
              {:else if r}<span class="ico" class:ok={r.ok} class:bad={!r.ok}><Icon name={r.ok ? "check" : "x"} size={10} /></span>
              {:else}<span class="ico pending"></span>{/if}
            </span>
            <span class="fname">{shortName(f)}</span>
            {#if r}
              <span class="bar"><span style:width="{(r.durationMs / maxDur) * 100}%" class:bad={!r.ok}></span></span>
              <span class="subtle t">{ms(r.durationMs)}</span>
            {/if}
          </button>
        </li>
      {:else}
        <li class="subtle none">{run.running ? "Starting…" : "No .hurl files were found in this folder."}</li>
      {/each}
    </ul>
    <div class="detail">
      {#if run.files.length}
        <FileRunView run={run.results[selected]} phase={run.states[selected]} runId={run.id} fileIndex={selected} />
      {/if}
    </div>
  </div>
</div>

<style>
  .report {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 14px 18px 10px;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 16px;
    font-weight: 600;
  }
  .stats {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .export {
    width: 96px;
  }
  .m {
    margin: 0 18px 8px;
  }
  .progress {
    height: 3px;
    background: var(--border-muted);
  }
  .progress div {
    height: 100%;
    background: var(--accent);
    transition: width 0.15s;
  }
  .split {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(220px, 34%) 1fr;
    border-top: 1px solid var(--border);
  }
  .files {
    list-style: none;
    margin: 0;
    padding: 4px;
    overflow: auto;
    border-right: 1px solid var(--border);
  }
  .file {
    display: grid;
    grid-template-columns: 18px 1fr 50px 52px;
    gap: 6px;
    align-items: center;
    width: 100%;
    padding: 5px 8px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    text-align: left;
    content-visibility: auto;
  }
  .file:hover {
    background: var(--row-hover);
  }
  .file.sel {
    background: var(--selection);
  }
  .fname {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }
  .t {
    font-size: 11px;
    text-align: right;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--border-muted);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--success);
  }
  .bar span.bad {
    background: var(--danger);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    color: #fff;
  }
  .ico.ok {
    background: var(--success-emphasis);
  }
  .ico.bad {
    background: var(--danger);
  }
  .ico.pending {
    border: 1.5px solid var(--border);
  }
  .spin {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .none {
    padding: 14px;
    font-size: 12px;
  }
  .detail {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
