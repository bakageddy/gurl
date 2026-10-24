<script lang="ts">
  import Icon from "../Icon.svelte";
  import QueryEditor from "./QueryEditor.svelte";
  import FiltersEditor from "./FiltersEditor.svelte";
  import { PREDICATES, predicateDef } from "../../lib/hurl-schema";
  import type { AssertM } from "../../lib/types";

  interface Props {
    assert: AssertM;
    ondelete: () => void;
    ontouch: () => void;
  }
  let { assert = $bindable(), ondelete, ontouch }: Props = $props();
  const takesValue = $derived(predicateDef(assert.func)?.value ?? true);
</script>

<div class="assert">
  <div class="line">
    <QueryEditor bind:query={assert.query} />
    <button class="icon-btn" title="Remove assert" onclick={ondelete}><Icon name="trash" size={13} /></button>
  </div>
  {#if assert.filters.length}
    <div class="filters"><FiltersEditor bind:filters={assert.filters} {ontouch} /></div>
  {/if}
  <div class="line pred">
    <label class="not" title="Negate the predicate"><input type="checkbox" bind:checked={assert.not} /> not</label>
    <select class="field" bind:value={assert.func} onchange={() => { if (!predicateDef(assert.func)?.value) assert.value = ""; }}>
      {#each PREDICATES as p}<option value={p.func}>{p.func} · {p.label}</option>{/each}
      {#if !predicateDef(assert.func)}<option value={assert.func}>{assert.func}</option>{/if}
    </select>
    {#if takesValue}
      <input class="field mono value" placeholder={'200, "text", true, /regex/, {{var}}'} bind:value={assert.value} spellcheck="false" />
    {/if}
    {#if !assert.filters.length}
      <button class="btn sm ghost" onclick={() => { assert.filters.push({ name: "count", args: "" }); ontouch(); }}>
        <Icon name="plus" size={12} /> filter
      </button>
    {/if}
  </div>
</div>

<style>
  .assert {
    display: grid;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    background: var(--canvas-subtle);
  }
  .line {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .line :global(.kind) {
    flex: none;
  }
  .filters {
    display: grid;
    gap: 4px;
    padding-left: 12px;
  }
  .pred select {
    width: 190px;
    flex: none;
  }
  .value {
    flex: 1;
    min-width: 100px;
  }
  .not {
    display: flex;
    gap: 4px;
    align-items: center;
    font-size: 12px;
    color: var(--fg-muted);
    flex: none;
  }
</style>
