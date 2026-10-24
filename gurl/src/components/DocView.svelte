<script lang="ts">
  import { setContext } from "svelte";
  import Icon from "./Icon.svelte";
  import Builder from "./builder/Builder.svelte";
  import TextEditor from "./LazyEditor.svelte";
  import FileRunView from "./run/FileRunView.svelte";
  import { settings } from "../lib/stores/settings.svelte";
  import { tabs, type DocTab } from "../lib/stores/tabs.svelte";
  import { runs } from "../lib/stores/runs.svelte";
  import { formatActive, runActive, saveActive } from "../lib/actions";
  import { ms } from "../lib/format";
  import { ASSERT_TARGET, type AssertTarget } from "../lib/assert-target";

  let { tab }: { tab: DocTab } = $props();

  let main = $state<HTMLDivElement>();
  let editor = $state<{ revealLine: (n: number) => void } | null>(null);

  // "Add assert" from this tab's own run result lands in this tab's text.
  setContext<AssertTarget>(ASSERT_TARGET, {
    enabled: () => !tab.run?.running,
    reason: () => "Wait for the run to finish",
    add: (entryIndex, items) => tab.addResponseItems(entryIndex, items),
  });

  $effect(() => {
    const line = tab.pendingReveal;
    if (line === null) return;
    if (tab.mode === "text" && editor) editor.revealLine(line);
    tab.pendingReveal = null;
  });

  async function setMode(mode: "builder" | "text") {
    await tab.sync();
    tab.mode = mode;
  }

  function startDrag(e: PointerEvent) {
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      const r = main!.getBoundingClientRect();
      const pct = ((r.right - ev.clientX) / r.width) * 100;
      settings.resultWidth = Math.min(75, Math.max(22, pct));
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      settings.save();
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  }

  const run = $derived(tab.run);
  const result = $derived(run?.results[0]);
</script>

<div class="doc">
  <div class="toolbar">
    <div class="segmented">
      <button class:active={tab.mode === "builder"} onclick={() => setMode("builder")}>Builder</button>
      <button class:active={tab.mode === "text"} onclick={() => setMode("text")}>Text</button>
    </div>
    {#if tab.path === null}<span class="badge warn">Unsaved scenario</span>{/if}
    <span class="spacer"></span>
    {#if tab.mode === "text"}
      <button class="btn" onclick={formatActive} title="Format with hurlfmt (Shift+Alt+F)"><Icon name="list" size={13} /> Format</button>
    {/if}
    <button class="btn" onclick={saveActive} disabled={!tab.dirty && tab.path !== null}><Icon name="save" size={13} /> Save</button>
    {#if run?.running}
      <button class="btn danger" onclick={() => runs.cancel(run)}><Icon name="stop" size={12} /> Stop</button>
    {:else}
      <button class="btn primary" onclick={runActive} title="Run (Ctrl/Cmd+Enter)"><Icon name="play" size={12} /> Run</button>
    {/if}
  </div>

  <div class="main" bind:this={main} style:grid-template-columns="1fr 6px {settings.resultWidth}%">
    <div class="editor-pane">
      {#if tab.mode === "builder"}
        <div class="scroll"><Builder {tab} /></div>
      {:else}
        <TextEditor bind:this={editor} value={tab.text} diagnostics={tab.diagnostics} onchange={(v) => tab.setText(v)} />
      {/if}
    </div>
    <div class="splitter" role="separator" aria-orientation="vertical" onpointerdown={startDrag}></div>
    <div class="result-pane">
      {#if !run}
        <div class="empty">
          <Icon name="play" size={22} />
          <p>Run this scenario to see the response, asserts, captures and timings.</p>
          <p class="subtle">Ctrl/Cmd + Enter</p>
        </div>
      {:else}
        <div class="res-head">
          {#if run.running}
            <span class="badge warn">Running…</span>
          {:else if result}
            <span class="badge" class:ok={result.ok} class:fail={!result.ok}>{result.ok ? "Passed" : "Failed"}</span>
            <span class="muted">{ms(result.durationMs)}</span>
          {/if}
          <span class="spacer"></span>
          {#if !run.running && run.id}
            <button class="btn sm ghost" onclick={() => tabs.openReport(run, `Run · ${tab.name}`)}><Icon name="external" size={12} /> Open report</button>
          {/if}
        </div>
        <div class="res-body">
          <FileRunView run={result} phase={run.states[0]} runId={run.id} fileIndex={0} />
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .doc {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--canvas);
  }
  .spacer {
    flex: 1;
  }
  .main {
    flex: 1;
    min-height: 0;
    display: grid;
  }
  .editor-pane {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .scroll {
    flex: 1;
    overflow: auto;
  }
  .splitter {
    cursor: col-resize;
    background: var(--border);
    background-clip: content-box;
    padding: 0 2.5px;
  }
  .splitter:hover {
    background-color: var(--accent);
  }
  .result-pane {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--canvas);
  }
  .res-head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .res-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .empty {
    display: grid;
    gap: 6px;
    place-content: center;
    justify-items: center;
    height: 100%;
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
  }
  .empty p {
    margin: 0;
    max-width: 280px;
    font-size: 12.5px;
  }
</style>
