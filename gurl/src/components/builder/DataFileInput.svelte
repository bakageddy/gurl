<script lang="ts">
  // A reference to a file that must live inside the scenario's directory (or below), shown with
  // a picker and a live status (missing / outside the allowed directory).
  import Icon from "../Icon.svelte";
  import { checkDataFile, pickDataFile } from "../../lib/api";
  import { ui } from "../../lib/stores/ui.svelte";

  interface Props {
    value: string;
    /** Absolute path of the .hurl file (null until the scenario has been saved). */
    hurlPath: string | null;
    onchange: () => void;
  }
  let { value = $bindable(), hurlPath, onchange }: Props = $props();

  let status = $state<{ exists: boolean; allowed: boolean } | null>(null);
  let seq = 0;

  $effect(() => {
    const ref = value;
    const path = hurlPath;
    const mine = ++seq;
    if (!path || !ref.trim() || ref.includes("{{")) {
      status = null;
      return;
    }
    checkDataFile(path, ref.trim())
      .then((s) => mine === seq && (status = s))
      .catch(() => mine === seq && (status = null));
  });

  async function choose() {
    if (!hurlPath) {
      ui.toast("Save the scenario first: data files are resolved relative to it.", "error");
      return;
    }
    try {
      const rel = await pickDataFile(hurlPath);
      if (rel !== null) {
        value = rel;
        onchange();
      }
    } catch (e) {
      ui.error(e);
    }
  }
</script>

<div class="wrap">
  <input class="field mono" placeholder="data/file.png" bind:value spellcheck="false" />
  <button class="btn sm" onclick={choose} title="Choose a file inside this scenario's folder"><Icon name="folder-open" size={12} /> Choose…</button>
</div>
{#if status && !status.allowed}
  <div class="hint bad"><Icon name="alert" size={11} /> Must be inside this scenario's folder (or a sub-folder).</div>
{:else if status && !status.exists}
  <div class="hint warn"><Icon name="alert" size={11} /> File not found next to the scenario.</div>
{:else if !hurlPath}
  <div class="hint subtle">Save the scenario to pick files from its folder.</div>
{/if}

<style>
  .wrap {
    display: flex;
    gap: 6px;
  }
  .wrap input {
    flex: 1;
    min-width: 0;
  }
  .hint {
    display: flex;
    gap: 4px;
    align-items: center;
    font-size: 11.5px;
    margin-top: 3px;
  }
  .hint.bad {
    color: var(--danger);
  }
  .hint.warn {
    color: var(--attention);
  }
</style>
