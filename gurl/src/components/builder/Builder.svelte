<script lang="ts">
  import Icon from "../Icon.svelte";
  import EntryCard from "./EntryCard.svelte";
  import { newEntry } from "../../lib/hurl-schema";
  import type { DocTab } from "../../lib/stores/tabs.svelte";

  let { tab }: { tab: DocTab } = $props();
  const touch = () => tab.touch();

  function move(i: number, d: number) {
    const list = tab.model!.entries;
    const j = i + d;
    if (j < 0 || j >= list.length) return;
    [list[i], list[j]] = [list[j], list[i]];
    touch();
  }
  function duplicate(i: number) {
    const copy = JSON.parse(JSON.stringify($state.snapshot(tab.model!.entries[i])));
    tab.model!.entries.splice(i + 1, 0, copy);
    touch();
  }
  function remove(i: number) {
    tab.model!.entries.splice(i, 1);
    touch();
  }
  function add() {
    tab.model!.entries.push(newEntry());
    touch();
  }
  function addFirst() {
    tab.model!.entries.unshift(newEntry());
    touch();
  }
</script>

<!-- input/change events from any nested form control bubble here and refresh the file text -->
<div class="builder" oninput={touch} onchange={touch}>
  {#if tab.diagnostics.length && !tab.model}
    <div class="banner error">
      <Icon name="alert" size={14} />
      <span>This file has a syntax error, so the builder can't show it yet: {tab.diagnostics[0].message.split("\n")[0]} (line {tab.diagnostics[0].line}).</span>
      <button class="btn sm" onclick={() => (tab.mode = "text")}>Fix in text view</button>
    </div>
  {:else if tab.model}
    {#if tab.diagnostics.length}
      <div class="banner error">
        <Icon name="alert" size={14} />
        <span>The text has a syntax error (line {tab.diagnostics[0].line}); showing the last valid version.</span>
        <button class="btn sm" onclick={() => (tab.mode = "text")}>Open text view</button>
      </div>
    {:else if !tab.lossless}
      <div class="banner warn">
        <Icon name="alert" size={14} />
        <span>Some formatting in this file can't be represented here and will be normalised when you edit with the builder.</span>
      </div>
    {:else if tab.droppedComments > 0}
      <div class="banner warn">
        <Icon name="alert" size={14} />
        <span>{tab.droppedComments} inline comment{tab.droppedComments === 1 ? "" : "s"} will be removed if you edit with the builder. Edit in the text view to keep them.</span>
      </div>
    {/if}

    {#if tab.model.entries.length > 0}
      <div class="top"><button class="btn sm add-top" onclick={addFirst} title="Insert a request before the first one"><Icon name="plus" size={12} /> Add request at top</button></div>
    {/if}
    {#each tab.model.entries as _, i (i)}
      <EntryCard
        bind:entry={tab.model.entries[i]}
        index={i}
        count={tab.model.entries.length}
        hurlPath={tab.path}
        onmove={(d) => move(i, d)}
        onduplicate={() => duplicate(i)}
        ondelete={() => remove(i)}
        ontouch={touch}
      />
    {/each}
    <button class="btn add" onclick={add}><Icon name="plus" size={13} /> Add request</button>
  {/if}
</div>

<style>
  .top {
    display: flex;
    justify-content: flex-end;
    padding: 0 0 2px;
  }

  .builder {
    display: grid;
    gap: 12px;
    padding: 14px;
    align-content: start;
  }
  .add {
    justify-self: start;
  }
</style>
