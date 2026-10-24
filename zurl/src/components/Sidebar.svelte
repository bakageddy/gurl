<script lang="ts">
  import Icon from "./Icon.svelte";
  import { workspace, type Row } from "../lib/stores/workspace.svelte";
  import { tabs } from "../lib/stores/tabs.svelte";
  import { runs } from "../lib/stores/runs.svelte";
  import { ui, type MenuItem } from "../lib/stores/ui.svelte";
  import { basename, timeAgo, ms } from "../lib/format";
  import {
    deleteEntry,
    duplicateEntry,
    newFolder,
    newScenario,
    openWorkspaceDialog,
    renameEntry,
    runDirectory,
    runFilePath,
  } from "../lib/actions";

  const ROW_H = 26;
  let view = $state<"files" | "history">("files");
  let scroller = $state<HTMLDivElement>();
  let scrollTop = $state(0);
  let height = $state(600);

  const total = $derived(workspace.rows.length);
  const start = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - 8));
  const end = $derived(Math.min(total, Math.ceil((scrollTop + height) / ROW_H) + 8));
  const slice = $derived(workspace.rows.slice(start, end));

  function activate(r: Row) {
    workspace.selected = r.node.path;
    if (r.node.kind === "dir") void workspace.toggle(r.node.path);
    else if (r.node.kind === "env") tabs.openVars(r.node.path.slice(0, r.node.path.length - r.node.name.length - 1));
    else void tabs.openFile(r.node.path);
  }

  function context(e: MouseEvent, r: Row) {
    workspace.selected = r.node.path;
    const p = r.node.path;
    let items: MenuItem[];
    if (r.node.kind === "dir") {
      items = [
        { label: "New scenario…", action: () => newScenario(p) },
        { label: "New folder…", action: () => newFolder(p) },
        { label: "Variables…", action: () => tabs.openVars(p) },
        { label: "", action: () => {}, separator: true },
        { label: "Run this folder", action: () => runDirectory(p, false) },
        { label: "Run folder recursively", action: () => runDirectory(p, true) },
        { label: "", action: () => {}, separator: true },
        { label: "Rename…", action: () => renameEntry(p) },
        { label: "Delete", action: () => deleteEntry(p), danger: true },
      ];
    } else if (r.node.kind === "hurl") {
      items = [
        { label: "Open", action: () => tabs.openFile(p) },
        { label: "Run", action: () => runFilePath(p) },
        { label: "Duplicate", action: () => duplicateEntry(p) },
        { label: "", action: () => {}, separator: true },
        { label: "Rename…", action: () => renameEntry(p) },
        { label: "Delete", action: () => deleteEntry(p), danger: true },
      ];
    } else {
      items = [{ label: "Delete variables file", action: () => deleteEntry(p), danger: true }];
    }
    ui.openMenu(e, items);
  }

  function rootMenu(e: MouseEvent) {
    if (!workspace.root) return;
    ui.openMenu(e, [
      { label: "New scenario…", action: () => newScenario(workspace.root) },
      { label: "New folder…", action: () => newFolder(workspace.root) },
      { label: "Variables…", action: () => tabs.openVars(workspace.root!) },
      { label: "Run workspace", action: () => runDirectory(workspace.root!, true) },
    ]);
  }
</script>

<aside class="sidebar">
  <div class="head">
    <div class="segmented">
      <button class:active={view === "files"} onclick={() => (view = "files")}>Files</button>
      <button class:active={view === "history"} onclick={() => { view = "history"; void runs.refreshHistory(); }}>History</button>
    </div>
    <div class="head-actions">
      <button class="icon-btn" title="New scenario" disabled={!workspace.root} onclick={() => newScenario()}><Icon name="file-plus" size={15} /></button>
      <button class="icon-btn" title="New folder" disabled={!workspace.root} onclick={() => newFolder()}><Icon name="folder-plus" size={15} /></button>
      <button class="icon-btn" title="Open workspace" onclick={openWorkspaceDialog}><Icon name="folder-open" size={15} /></button>
    </div>
  </div>

  {#if view === "files"}
    {#if workspace.root}
      <button class="root" oncontextmenu={rootMenu} onclick={() => (workspace.selected = workspace.root)} title={workspace.root}>
        <Icon name="folder-open" size={13} />
        <span>{basename(workspace.root)}</span>
        <span class="subtle mono">{total}</span>
      </button>
      <div
        class="tree"
        bind:this={scroller}
        bind:clientHeight={height}
        onscroll={() => (scrollTop = scroller!.scrollTop)}
      >
        <div style:height="{total * ROW_H}px" style:position="relative">
          {#each slice as r, i (r.node.path)}
            <div
              class="row"
              class:selected={workspace.selected === r.node.path}
              class:active={tabs.active?.id === r.node.path}
              style:top="{(start + i) * ROW_H}px"
              style:padding-left="{8 + r.depth * 14}px"
              role="treeitem"
              aria-selected={workspace.selected === r.node.path}
              tabindex="-1"
              onclick={() => activate(r)}
              onkeydown={(e) => e.key === "Enter" && activate(r)}
              oncontextmenu={(e) => context(e, r)}
            >
              {#if r.node.kind === "dir"}
                <Icon name={workspace.expanded[r.node.path] ? "chevron-down" : "chevron-right"} size={11} class="chev" />
                <Icon name={workspace.expanded[r.node.path] ? "folder-open" : "folder"} size={14} class="ic dir" />
              {:else if r.node.kind === "env"}
                <span class="chev"></span><Icon name="key" size={14} class="ic env" />
              {:else}
                <span class="chev"></span><Icon name="file" size={14} class="ic" />
              {/if}
              <span class="name">{r.node.name}</span>
              {#if r.node.kind === "dir" && r.node.hasEnv}
                <span class="env-dot" title="Has variables (zurl.env)"></span>
              {/if}
            </div>
          {/each}
        </div>
        {#if total === 0}
          <p class="empty subtle">This folder has no scenarios yet.<br />Create one with the buttons above.</p>
        {/if}
      </div>
    {:else}
      <div class="welcome">
        <Icon name="folder" size={28} />
        <p>Open a folder to use it as a workspace. Every folder can hold scenarios and a <code>zurl.env</code> with variables for itself and its sub-folders.</p>
        <button class="btn primary" onclick={openWorkspaceDialog}>Open folder…</button>
      </div>
    {/if}
  {:else}
    <div class="history">
      {#each runs.history as h (h.id)}
        <button class="hist" onclick={async () => tabs.openReport(await runs.open(h), `Run · ${basename(h.label)}`)}>
          <span class="dot" class:ok={h.failed === 0} class:fail={h.failed > 0}></span>
          <span class="h-main">
            <span class="h-title">{basename(h.label)}</span>
            <span class="subtle h-sub">{h.passed}/{h.total} passed · {ms(h.durationMs)} · {timeAgo(h.startedAt)}</span>
          </span>
        </button>
      {:else}
        <p class="empty subtle">No runs yet.</p>
      {/each}
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
  }
  :global(html.platform-mac) .sidebar {
    background: color-mix(in srgb, var(--sidebar-bg) 55%, transparent);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px 6px;
    gap: 6px;
  }
  .head-actions {
    display: flex;
  }
  .head-actions .icon-btn:disabled {
    opacity: 0.35;
  }
  .root {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 2px 8px 4px;
    padding: 4px 6px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    font-weight: 600;
    text-align: left;
  }
  .root span:nth-child(2) {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tree {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 6px 8px;
  }
  .row {
    position: absolute;
    left: 0;
    right: 0;
    height: 26px;
    display: flex;
    align-items: center;
    gap: 5px;
    border-radius: var(--radius);
    white-space: nowrap;
    cursor: default;
  }
  .row:hover {
    background: var(--row-hover);
  }
  .row.selected {
    background: var(--selection);
  }
  .row.active .name {
    font-weight: 600;
  }
  .row :global(.chev) {
    width: 11px;
    flex: none;
    color: var(--fg-subtle);
  }
  .row :global(.ic) {
    flex: none;
    color: var(--fg-muted);
  }
  .row :global(.ic.dir) {
    color: var(--accent);
  }
  .row :global(.ic.env) {
    color: var(--attention);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .env-dot {
    width: 6px;
    height: 6px;
    margin-left: auto;
    margin-right: 8px;
    border-radius: 50%;
    background: var(--attention);
    flex: none;
  }
  .empty {
    padding: 16px;
    text-align: center;
    font-size: 12px;
  }
  .welcome {
    display: grid;
    gap: 12px;
    justify-items: center;
    text-align: center;
    padding: 28px 18px;
    color: var(--fg-muted);
  }
  .welcome p {
    margin: 0;
    font-size: 12px;
  }
  .history {
    flex: 1;
    overflow: auto;
    padding: 4px 8px;
  }
  .hist {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    width: 100%;
    padding: 6px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    text-align: left;
  }
  .hist:hover {
    background: var(--row-hover);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-top: 6px;
    background: var(--fg-subtle);
    flex: none;
  }
  .dot.ok {
    background: var(--success);
  }
  .dot.fail {
    background: var(--danger);
  }
  .h-main {
    display: grid;
    min-width: 0;
  }
  .h-title {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .h-sub {
    font-size: 11.5px;
  }
</style>
