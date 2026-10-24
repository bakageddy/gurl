<script lang="ts">
  import Icon from "../Icon.svelte";
  import ResponseView from "./ResponseView.svelte";
  import { ms } from "../../lib/format";
  import type { FileRun } from "../../lib/types";

  interface Props {
    run: FileRun | undefined;
    phase?: "pending" | "running" | "done";
    runId: string;
    fileIndex: number;
  }
  let { run, phase = "done", runId, fileIndex }: Props = $props();

  let selected = $state(0);
  const entries = $derived(run?.report?.entries ?? []);
  const entry = $derived(entries[Math.min(selected, Math.max(entries.length - 1, 0))]);
  const lastCall = $derived(entry?.calls[entry.calls.length - 1]);
  const entryOk = (e: (typeof entries)[number]) => e.asserts.every((a) => a.success);

  $effect(() => {
    run;
    selected = 0;
  });
</script>

<div class="fr">
{#if !run}
  <div class="state subtle">{phase === "running" ? "Running…" : "Waiting…"}</div>
{:else}
  {#if run.error}
    <div class="banner error err selectable">
      <Icon name="alert" size={14} />
      <pre class="mono">{run.error}</pre>
    </div>
  {/if}
  {#if entries.length}
    <div class="entries" role="listbox">
      {#each entries as e, i (i)}
        {@const c = e.calls[e.calls.length - 1]}
        <button class="entry" class:sel={selected === i} onclick={() => (selected = i)}>
          <span class="ico" class:ok={entryOk(e)} class:bad={!entryOk(e)}><Icon name={entryOk(e) ? "check" : "x"} size={11} /></span>
          <span class="m mono">{c?.request.method}</span>
          <span class="u mono">{c?.request.url}</span>
          <span class="s">{c?.response.status ?? "—"}</span>
          <span class="t subtle">{ms(e.time / 1000)}</span>
        </button>
      {/each}
    </div>
    {#if entry && lastCall}
      <div class="detail">
        <ResponseView {entry} call={lastCall} {runId} {fileIndex} entryIndex={selected} callIndex={entry.calls.length - 1} />
      </div>
    {/if}
  {:else if !run.error}
    <div class="state subtle">No requests were executed.</div>
  {/if}
  <div class="foot subtle">{run.ok ? "Passed" : "Failed"} in {ms(run.durationMs)}</div>
{/if}
</div>

<style>
  .state {
    display: grid;
    place-items: center;
    height: 100%;
    font-size: 12.5px;
  }
  .err {
    margin: 10px;
    align-items: flex-start;
  }
  .err pre {
    margin: 0;
    white-space: pre-wrap;
    font-size: 11.5px;
  }
  .entries {
    max-height: 32%;
    overflow: auto;
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .entry {
    display: grid;
    grid-template-columns: 18px 56px 1fr 40px 64px;
    gap: 8px;
    align-items: center;
    width: 100%;
    padding: 5px 12px;
    border: 0;
    background: transparent;
    text-align: left;
    font-size: 12px;
  }
  .entry:hover {
    background: var(--row-hover);
  }
  .entry.sel {
    background: var(--selection);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
  }
  .ico.ok {
    color: #fff;
    background: var(--success-emphasis);
  }
  .ico.bad {
    color: #fff;
    background: var(--danger);
  }
  .m {
    font-weight: 700;
    color: var(--fg-muted);
  }
  .u {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .t {
    text-align: right;
  }
  .detail {
    flex: 1;
    min-height: 0;
  }
  .foot {
    padding: 4px 12px;
    font-size: 11.5px;
    border-top: 1px solid var(--border-muted);
    flex: none;
  }
  .fr {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
</style>
