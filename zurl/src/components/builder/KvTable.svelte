<script lang="ts">
  import Icon from "../Icon.svelte";
  import type { Kv } from "../../lib/types";

  interface Props {
    items: Kv[];
    namePlaceholder?: string;
    valuePlaceholder?: string;
    suggest?: string[];
    listId?: string;
    ontouch: () => void;
    addLabel?: string;
  }
  let {
    items = $bindable(),
    namePlaceholder = "name",
    valuePlaceholder = "value",
    suggest = [],
    listId = "kv-suggest",
    ontouch,
    addLabel = "Add",
  }: Props = $props();

  function add() {
    items.push({ name: "", value: "" });
    ontouch();
  }
  function remove(i: number) {
    items.splice(i, 1);
    ontouch();
  }
</script>

<div class="kv">
  {#each items as item, i (i)}
    <div class="row">
      <input class="field mono" list={suggest.length ? listId : undefined} placeholder={namePlaceholder} bind:value={item.name} spellcheck="false" />
      <input class="field mono" placeholder={valuePlaceholder} bind:value={item.value} spellcheck="false" />
      <button class="icon-btn" title="Remove" onclick={() => remove(i)}><Icon name="x" size={13} /></button>
    </div>
  {/each}
  {#if suggest.length}
    <datalist id={listId}>{#each suggest as s}<option value={s}></option>{/each}</datalist>
  {/if}
  <button class="btn sm ghost add" onclick={add}><Icon name="plus" size={12} /> {addLabel}</button>
</div>

<style>
  .kv {
    display: grid;
    gap: 6px;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(120px, 0.8fr) 1.6fr 24px;
    gap: 6px;
    align-items: center;
  }
  .add {
    justify-self: start;
  }
</style>
