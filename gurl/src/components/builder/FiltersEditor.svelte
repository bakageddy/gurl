<script lang="ts">
  import Icon from "../Icon.svelte";
  import { FILTERS, filterDef } from "../../lib/hurl-schema";
  import type { FilterM } from "../../lib/types";

  let { filters = $bindable(), ontouch }: { filters: FilterM[]; ontouch: () => void } = $props();
</script>

{#each filters as f, i (i)}
  <div class="filter">
    <Icon name="chevron-right" size={11} class="arrow" />
    <select class="field" bind:value={f.name} onchange={() => { if (!filterDef(f.name)?.arg) f.args = ""; }}>
      {#each FILTERS as d}<option value={d.name}>{d.name}</option>{/each}
      {#if !filterDef(f.name)}<option value={f.name}>{f.name}</option>{/if}
    </select>
    {#if filterDef(f.name)?.arg !== null}
      <input class="field mono" placeholder={filterDef(f.name)?.arg ?? ""} bind:value={f.args} spellcheck="false" />
    {/if}
    <button class="icon-btn" title="Remove filter" onclick={() => { filters.splice(i, 1); ontouch(); }}><Icon name="x" size={12} /></button>
  </div>
{/each}
<button class="btn sm ghost" onclick={() => { filters.push({ name: "count", args: "" }); ontouch(); }}>
  <Icon name="plus" size={12} /> filter
</button>

<style>
  .filter {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .filter :global(.arrow) {
    color: var(--fg-subtle);
    flex: none;
  }
  .filter select {
    width: 150px;
    flex: none;
  }
  .filter input {
    flex: 1;
    min-width: 80px;
  }
</style>
