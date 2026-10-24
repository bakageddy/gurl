<script lang="ts">
  import Icon from "../Icon.svelte";
  import DataFileInput from "./DataFileInput.svelte";
  import type { MultipartM } from "../../lib/types";

  interface Props {
    parts: MultipartM[];
    hurlPath: string | null;
    ontouch: () => void;
  }
  let { parts = $bindable(), hurlPath, ontouch }: Props = $props();

  function add(kind: "param" | "file") {
    parts.push({ name: "", kind, value: "", contentType: kind === "file" ? "" : null });
    ontouch();
  }
</script>

<div class="mp">
  <p class="note subtle">
    Sent as <code>multipart/form-data</code>. File parts are read from this scenario's folder (or a sub-folder) and
    can't point outside it.
  </p>
  {#each parts as p, i (i)}
    <div class="part">
      <input class="field mono" placeholder="field name" bind:value={p.name} spellcheck="false" />
      <select
        class="field"
        bind:value={p.kind}
        onchange={() => {
          p.contentType = p.kind === "file" ? "" : null;
          ontouch();
        }}
      >
        <option value="param">Text</option>
        <option value="file">File</option>
      </select>
      <button class="icon-btn" title="Remove part" onclick={() => { parts.splice(i, 1); ontouch(); }}><Icon name="x" size={13} /></button>
      <div class="value">
        {#if p.kind === "file"}
          <DataFileInput bind:value={p.value} {hurlPath} onchange={ontouch} />
          <input class="field mono ct" placeholder="content type (optional), e.g. image/png" bind:value={p.contentType} spellcheck="false" />
        {:else}
          <input class="field mono" placeholder="value" bind:value={p.value} spellcheck="false" />
        {/if}
      </div>
    </div>
  {/each}
  <div class="adds">
    <button class="btn sm ghost" onclick={() => add("param")}><Icon name="plus" size={12} /> Text field</button>
    <button class="btn sm ghost" onclick={() => add("file")}><Icon name="plus" size={12} /> File</button>
  </div>
</div>

<style>
  .mp {
    display: grid;
    gap: 8px;
  }
  .note {
    margin: 0;
    font-size: 12px;
  }
  .part {
    display: grid;
    grid-template-columns: minmax(120px, 0.7fr) 90px 24px;
    gap: 6px;
    align-items: center;
    padding: 8px;
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    background: var(--canvas-subtle);
  }
  .value {
    grid-column: 1 / -1;
    display: grid;
    gap: 6px;
  }
  .adds {
    display: flex;
    gap: 6px;
  }
</style>
