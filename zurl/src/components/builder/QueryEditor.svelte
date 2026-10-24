<script lang="ts">
  import { QUERY_GROUPS, QUERY_KINDS, queryKind } from "../../lib/hurl-schema";
  import type { QueryM } from "../../lib/types";

  let { query = $bindable() }: { query: QueryM } = $props();
  const def = $derived(queryKind(query.kind));

  function changeKind() {
    if (!queryKind(query.kind)?.arg) query.args = "";
  }
</script>

<select class="field kind" bind:value={query.kind} onchange={changeKind}>
  {#each QUERY_GROUPS as g}
    <optgroup label={g.label}>
      {#each QUERY_KINDS.filter((q) => q.group === g.id) as q}
        <option value={q.kind}>{q.label}</option>
      {/each}
    </optgroup>
  {/each}
  {#if !def}<option value={query.kind}>{query.kind}</option>{/if}
</select>
{#if def?.arg !== null}
  <input class="field mono arg" placeholder={def?.arg ?? "argument"} bind:value={query.args} spellcheck="false" />
{/if}

<style>
  .kind {
    width: 118px;
    flex: none;
  }
  .arg {
    flex: 1;
    min-width: 90px;
  }
</style>
