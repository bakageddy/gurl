<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import { nativeChrome } from "../lib/platform";
  import { workspace } from "../lib/stores/workspace.svelte";
  import { tabs } from "../lib/stores/tabs.svelte";
  import { basename } from "../lib/format";

  const win = getCurrentWindow();
  const title = $derived(
    [workspace.root ? basename(workspace.root) : null, tabs.active && "title" in tabs.active ? tabs.active.title : null]
      .filter(Boolean)
      .join("  —  ") || "gURL",
  );
</script>

<header class="titlebar" role="presentation" class:native={nativeChrome} data-tauri-drag-region ondblclick={() => !nativeChrome && win.toggleMaximize()}>
  {#if !nativeChrome}
    <div class="lights">
      <button class="light close" aria-label="Close" onclick={() => win.close()}><Icon name="x" size={8} /></button>
      <button class="light min" aria-label="Minimize" onclick={() => win.minimize()}><span></span></button>
      <button class="light max" aria-label="Maximize" onclick={() => win.toggleMaximize()}><Icon name="plus" size={8} /></button>
    </div>
  {:else}
    <div class="native-gap" data-tauri-drag-region></div>
  {/if}
  <div class="title" data-tauri-drag-region>{title}</div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: var(--titlebar-h);
    padding: 0 10px;
    background: var(--titlebar-bg);
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .titlebar.native {
    background: transparent;
  }
  .native-gap {
    width: 74px;
    height: 100%;
  }
  .lights {
    display: flex;
    gap: 8px;
    padding: 0 6px 0 4px;
  }
  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 0.5px solid rgba(0, 0, 0, 0.25);
    padding: 0;
    display: grid;
    place-items: center;
    color: transparent;
  }
  .light.close {
    background: #ff5f57;
  }
  .light.min {
    background: #febc2e;
  }
  .light.max {
    background: #28c840;
  }
  .lights:hover .light {
    color: rgba(0, 0, 0, 0.55);
  }
  .light.min span {
    width: 6px;
    height: 1.4px;
    background: currentColor;
  }
  .title {
    flex: 1;
    text-align: center;
    font-weight: 600;
    font-size: 12.5px;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    pointer-events: none;
  }
</style>
