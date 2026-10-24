<script lang="ts">
  import { ui } from "../lib/stores/ui.svelte";

  let promptValue = $state("");
  let inputEl = $state<HTMLInputElement>();

  $effect(() => {
    if (ui.prompt) {
      promptValue = ui.prompt.value ?? "";
      queueMicrotask(() => {
        inputEl?.focus();
        inputEl?.select();
      });
    }
  });

  function closePrompt(v: string | null) {
    const p = ui.prompt;
    ui.prompt = null;
    p?.resolve(v);
  }
  function closeConfirm(v: boolean) {
    const c = ui.confirmation;
    ui.confirmation = null;
    c?.resolve(v);
  }
</script>

<svelte:window
  onclick={() => (ui.menu = null)}
  onblur={() => (ui.menu = null)}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      ui.menu = null;
      if (ui.prompt) closePrompt(null);
      if (ui.confirmation) closeConfirm(false);
    }
  }}
/>

{#if ui.menu}
  <div class="menu" style:left="{Math.min(ui.menu.x, innerWidth - 200)}px" style:top="{Math.min(ui.menu.y, innerHeight - ui.menu.items.length * 28 - 16)}px" role="menu">
    {#each ui.menu.items as item}
      {#if item.separator}
        <hr />
      {:else}
        <button role="menuitem" class:danger={item.danger} disabled={item.disabled} onclick={() => { ui.menu = null; item.action(); }}>{item.label}</button>
      {/if}
    {/each}
  </div>
{/if}

{#if ui.prompt}
  <div class="scrim" role="presentation" onclick={() => closePrompt(null)}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
    <form class="dialog" aria-label={ui.prompt.title} onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} onsubmit={(e) => { e.preventDefault(); closePrompt(promptValue); }}>
      <h3>{ui.prompt.title}</h3>
      {#if ui.prompt.label}<label for="prompt-input">{ui.prompt.label}</label>{/if}
      <input id="prompt-input" class="field" bind:this={inputEl} bind:value={promptValue} spellcheck="false" />
      <div class="buttons">
        <button type="button" class="btn" onclick={() => closePrompt(null)}>Cancel</button>
        <button type="submit" class="btn primary" disabled={!promptValue.trim()}>{ui.prompt.confirmLabel ?? "OK"}</button>
      </div>
    </form>
  </div>
{/if}

{#if ui.confirmation}
  <div class="scrim" role="presentation" onclick={() => closeConfirm(false)}>
    <div class="dialog" role="alertdialog" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <h3>{ui.confirmation.title}</h3>
      <p>{ui.confirmation.message}</p>
      <div class="buttons">
        <button class="btn" onclick={() => closeConfirm(false)}>Cancel</button>
        <button class="btn" class:primary={!ui.confirmation.danger} class:danger={ui.confirmation.danger} onclick={() => closeConfirm(true)}>
          {ui.confirmation.confirmLabel ?? "OK"}
        </button>
      </div>
    </div>
  </div>
{/if}

<div class="toasts" aria-live="polite">
  {#each ui.toasts as t (t.id)}
    <div class="toast" class:error={t.kind === "error"}>{t.text}</div>
  {/each}
</div>

<style>
  .menu {
    position: fixed;
    z-index: 900;
    min-width: 188px;
    padding: 4px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--canvas-overlay);
    box-shadow: var(--shadow);
  }
  .menu button {
    display: block;
    width: 100%;
    padding: 4px 10px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    text-align: left;
    font-size: 12.5px;
  }
  .menu button:hover:not(:disabled) {
    background: var(--accent-emphasis);
    color: #fff;
  }
  .menu button.danger {
    color: var(--danger);
  }
  .menu button.danger:hover {
    background: var(--danger);
    color: #fff;
  }
  hr {
    border: 0;
    border-top: 1px solid var(--border-muted);
    margin: 4px 2px;
  }
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 800;
    display: grid;
    place-items: center;
    background: rgba(1, 4, 9, 0.4);
  }
  .dialog {
    display: grid;
    gap: 8px;
    width: min(420px, calc(100vw - 32px));
    padding: 18px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--canvas-overlay);
    box-shadow: var(--shadow);
  }
  h3 {
    margin: 0;
    font-size: 14px;
  }
  p {
    margin: 0;
    color: var(--fg-muted);
  }
  label {
    font-size: 12px;
    color: var(--fg-muted);
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }
  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 950;
    display: grid;
    gap: 8px;
  }
  .toast {
    max-width: 420px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--canvas-overlay);
    box-shadow: var(--shadow);
    font-size: 12.5px;
    user-select: text;
  }
  .toast.error {
    border-color: color-mix(in srgb, var(--danger) 50%, transparent);
    color: var(--danger);
  }
</style>
