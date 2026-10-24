<script lang="ts">
  import Icon from "../Icon.svelte";
  import QueryEditor from "./QueryEditor.svelte";
  import FiltersEditor from "./FiltersEditor.svelte";
  import type { CaptureM } from "../../lib/types";

  interface Props {
    capture: CaptureM;
    ondelete: () => void;
    ontouch: () => void;
  }
  let { capture = $bindable(), ondelete, ontouch }: Props = $props();
</script>

<div class="capture">
  <div class="line">
    <input class="field mono name" placeholder="variable name" bind:value={capture.name} spellcheck="false" />
    <span class="eq">←</span>
    <QueryEditor bind:query={capture.query} />
    <label class="redact" title="Hide this value in logs and reports"><input type="checkbox" bind:checked={capture.redacted} /> redact</label>
    <button class="icon-btn" title="Remove capture" onclick={ondelete}><Icon name="trash" size={13} /></button>
  </div>
  <div class="filters"><FiltersEditor bind:filters={capture.filters} {ontouch} /></div>
</div>

<style>
  .capture {
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
  .name {
    width: 150px;
    flex: none;
  }
  .eq {
    color: var(--fg-subtle);
  }
  .redact {
    display: flex;
    gap: 4px;
    align-items: center;
    font-size: 12px;
    color: var(--fg-muted);
    flex: none;
  }
  .filters {
    display: grid;
    gap: 4px;
    padding-left: 12px;
  }
</style>
