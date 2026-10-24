<script lang="ts">
  import Icon from "../Icon.svelte";
  import { OPTION_DEFS, optionDef } from "../../lib/hurl-schema";
  import type { Kv } from "../../lib/types";

  let { options = $bindable(), ontouch }: { options: Kv[]; ontouch: () => void } = $props();

  const groups = [...new Set(OPTION_DEFS.map((o) => o.group))];
  let adding = $state("");

  function add(name: string) {
    const def = optionDef(name);
    if (!def) return;
    options.push({ name, value: def.type === "bool" ? "true" : def.type === "verbosity" ? "brief" : "" });
    adding = "";
    ontouch();
  }
</script>

<div class="opts">
  {#each options as o, i (i)}
    {@const def = optionDef(o.name)}
    <div class="row">
      <span class="name" title={o.name}>{def?.label ?? o.name} <code class="subtle">{o.name}</code></span>
      {#if def?.type === "bool"}
        <select class="field" bind:value={o.value}>
          <option value="true">true</option>
          <option value="false">false</option>
        </select>
      {:else if def?.type === "verbosity"}
        <select class="field" bind:value={o.value}>
          <option value="brief">brief</option>
          <option value="verbose">verbose</option>
          <option value="debug">debug</option>
        </select>
      {:else}
        <input class="field mono" placeholder={def?.hint ?? ""} bind:value={o.value} spellcheck="false" />
      {/if}
      <button class="icon-btn" title="Remove option" onclick={() => { options.splice(i, 1); ontouch(); }}><Icon name="x" size={13} /></button>
    </div>
  {/each}
  <select class="field add" bind:value={adding} onchange={() => adding && add(adding)}>
    <option value="">+ Add option…</option>
    {#each groups as g}
      <optgroup label={g}>
        {#each OPTION_DEFS.filter((o) => o.group === g) as d}<option value={d.name}>{d.label}</option>{/each}
      </optgroup>
    {/each}
  </select>
</div>

<style>
  .opts {
    display: grid;
    gap: 6px;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(150px, 0.9fr) 1.4fr 24px;
    gap: 8px;
    align-items: center;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name code {
    font-size: 11px;
    margin-left: 4px;
  }
  .add {
    width: 200px;
  }
</style>
