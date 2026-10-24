<script lang="ts">
  // The URL bar of a quick request: method, URL, Send. Pasting a curl command fills the whole tab.
  import Icon from "./Icon.svelte";
  import { importPreview } from "../lib/api";
  import { saveActive } from "../lib/actions";
  import { METHODS } from "../lib/hurl-schema";
  import { runs } from "../lib/stores/runs.svelte";
  import type { DocTab } from "../lib/stores/tabs.svelte";
  import { ui } from "../lib/stores/ui.svelte";

  let { tab }: { tab: DocTab } = $props();

  const req = $derived(tab.model?.entries[0]?.request);
  let input = $state<HTMLInputElement | null>(null);
  let focused = false;

  $effect(() => {
    if (input && !focused) {
      focused = true;
      input.focus();
      input.select();
    }
  });

  function send() {
    void tab.execute("text");
  }

  async function onPaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData("text") ?? "";
    if (!/^\s*(?:\S*[/\\])?curl(?:\.exe)?\s/.test(text)) return;
    e.preventDefault();
    try {
      const r = await importPreview("curl", { text }, true);
      const file = r.files[0];
      if (!file) return;
      tab.setText(file.text);
      await tab.sync();
      for (const w of r.warnings) ui.toast(w);
      ui.toast("cURL command imported");
    } catch (err) {
      ui.error(err);
    }
  }
</script>

{#if req}
  <div class="quick">
    <select class="field method mono" bind:value={req.method} onchange={() => tab.touch()}>
      {#each METHODS as m}<option value={m}>{m}</option>{/each}
    </select>
    <input
      class="field mono url"
      type="text"
      bind:this={input}
      bind:value={req.url}
      oninput={() => tab.touch()}
      onpaste={onPaste}
      onkeydown={(e) => e.key === "Enter" && send()}
      placeholder="https://api.example.com/path  — or paste a curl command"
      spellcheck="false"
      autocomplete="off"
    />
    {#if tab.run?.running}
      <button class="btn danger" onclick={() => tab.run && runs.cancel(tab.run)}><Icon name="stop" size={12} /> Stop</button>
    {:else}
      <button class="btn primary" onclick={send} title="Send (Enter)"><Icon name="play" size={12} /> Send</button>
    {/if}
    <button class="btn" onclick={saveActive} title="Save as a scenario in the workspace"><Icon name="save" size={13} /> Save as…</button>
  </div>
{/if}

<style>
  .quick {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--canvas-subtle);
  }
  .method {
    width: 104px;
    font-weight: 600;
  }
  .url {
    flex: 1;
    min-width: 0;
    font-size: 13px;
  }
</style>
