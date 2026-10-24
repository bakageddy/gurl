<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Icon from "./components/Icon.svelte";
  import Titlebar from "./components/Titlebar.svelte";
  import ResizeEdges from "./components/ResizeEdges.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import DocView from "./components/DocView.svelte";
  import VarsPanel from "./components/VarsPanel.svelte";
  import ReportView from "./components/run/ReportView.svelte";
  import Overlays from "./components/Overlays.svelte";
  import SettingsDialog from "./components/SettingsDialog.svelte";
  import ImportDialog from "./components/ImportDialog.svelte";
  import { nativeChrome } from "./lib/platform";
  import { settings } from "./lib/stores/settings.svelte";
  import { workspace } from "./lib/stores/workspace.svelte";
  import { tabs } from "./lib/stores/tabs.svelte";
  import { runs } from "./lib/stores/runs.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import { formatActive, newScenario, openFileDialog, openWorkspaceDialog, openWorkspacePath, runActive, saveActive } from "./lib/actions";

  let sidebarOpen = $state(true);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      await settings.load();
      void runs.refreshHistory();
      if (settings.lastWorkspace) await openWorkspacePath(settings.lastWorkspace);
      unlisten = await listen<string[]>("workspace-changed", async (e) => {
        await workspace.refresh(e.payload);
        for (const t of tabs.list) if (t.kind === "doc") void t.reloadFromDisk();
      });
    })();
    return () => unlisten?.();
  });

  function onKey(e: KeyboardEvent) {
    if (e.shiftKey && e.altKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      void formatActive();
      return;
    }
    const mod = e.ctrlKey || e.metaKey;
    if (!mod) return;
    if (e.key === "s") {
      e.preventDefault();
      void saveActive();
    } else if (e.key === "Enter") {
      e.preventDefault();
      void runActive();
    } else if (e.key === "o") {
      e.preventDefault();
      void (e.shiftKey ? openWorkspaceDialog() : openFileDialog());
    } else if (e.key === "n") {
      e.preventDefault();
      tabs.newUntitled();
    } else if (e.key === "w" && tabs.activeId) {
      e.preventDefault();
      void tabs.close(tabs.activeId);
    } else if (e.key === "b") {
      e.preventDefault();
      sidebarOpen = !sidebarOpen;
    }
  }

  function startSidebarDrag(e: PointerEvent) {
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => (settings.sidebarWidth = Math.min(520, Math.max(200, ev.clientX)));
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      settings.save();
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="app">
  <Titlebar />
  <div class="body" style:grid-template-columns="{sidebarOpen ? settings.sidebarWidth : 44}px 1fr">
    <div class="side">
      <Sidebar collapsed={!sidebarOpen} ontoggle={() => (sidebarOpen = !sidebarOpen)} />
      {#if sidebarOpen}
        <div class="grip" role="separator" aria-orientation="vertical" onpointerdown={startSidebarDrag}></div>
      {/if}
    </div>
    <main>
      <div class="tabbar" role="tablist">
        {#each tabs.list as t (t.id)}
          <div class="tab" class:active={tabs.activeId === t.id} role="tab" tabindex="0" aria-selected={tabs.activeId === t.id}
            onclick={() => (tabs.activeId = t.id)} onkeydown={(e) => e.key === "Enter" && (tabs.activeId = t.id)}
            onauxclick={(e) => e.button === 1 && tabs.close(t.id)}>
            <Icon name={t.kind === "doc" ? "file" : t.kind === "vars" ? "key" : "list"} size={13} />
            <span class="t-name">{t.title}</span>
            {#if t.kind === "doc" && t.dirty}<span class="dirty" title="Unsaved changes"></span>{/if}
            <button class="icon-btn close" aria-label="Close tab" onclick={(e) => { e.stopPropagation(); void tabs.close(t.id); }}><Icon name="x" size={11} /></button>
          </div>
        {/each}
        <button class="icon-btn newtab" title="New scenario (Ctrl/Cmd+N)" onclick={() => tabs.newUntitled()}><Icon name="plus" size={14} /></button>
      </div>
      <div class="content">
        {#if tabs.active?.kind === "doc"}
          {#key tabs.active.id}<DocView tab={tabs.active} />{/key}
        {:else if tabs.active?.kind === "vars"}
          {#key tabs.active.id}<VarsPanel dir={tabs.active.dir} />{/key}
        {:else if tabs.active?.kind === "report"}
          {#key tabs.active.id}<ReportView run={tabs.active.run} />{/key}
        {:else}
          <div class="welcome">
            <h1>gURL</h1>
            <p class="muted">A native-feeling workspace for HURL scenarios.</p>
            <div class="actions">
              <button class="btn primary" onclick={openWorkspaceDialog}><Icon name="folder-open" size={13} /> Open workspace</button>
              <button class="btn" onclick={() => tabs.newUntitled()}><Icon name="file-plus" size={13} /> New scenario</button>
              <button class="btn" onclick={openFileDialog}><Icon name="file" size={13} /> Open file</button>
              {#if workspace.root}<button class="btn" onclick={() => newScenario()}><Icon name="plus" size={13} /> Scenario in workspace</button>{/if}
            </div>
            <p class="subtle keys">Ctrl/Cmd+Shift+O workspace · Ctrl/Cmd+O file · Ctrl/Cmd+N new · Ctrl/Cmd+Enter run · Ctrl/Cmd+S save</p>
          </div>
        {/if}
      </div>
    </main>
  </div>
</div>

{#if !nativeChrome}<ResizeEdges />{/if}
<Overlays />
<SettingsDialog />
<ImportDialog />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: grid;
  }
  .side {
    position: relative;
    min-height: 0;
  }
  .grip {
    position: absolute;
    top: 0;
    right: -3px;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 5;
  }
  .grip:hover {
    background: color-mix(in srgb, var(--accent) 50%, transparent);
  }
  main {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--canvas);
  }
  .tabbar {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    padding: 6px 8px 0;
    border-bottom: 1px solid var(--border);
    background: var(--canvas-subtle);
    overflow-x: auto;
    flex: none;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 220px;
    height: 30px;
    padding: 0 6px 0 10px;
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: 8px 8px 0 0;
    color: var(--fg-muted);
    font-size: 12.5px;
  }
  .tab:hover {
    background: var(--row-hover);
  }
  .tab.active {
    background: var(--canvas);
    color: var(--fg);
    border-color: var(--border);
    margin-bottom: -1px;
    height: 31px;
  }
  .t-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dirty {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--attention);
    flex: none;
  }
  .close {
    width: 18px;
    height: 18px;
    opacity: 0.6;
  }
  .newtab {
    margin: 0 0 3px 4px;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .welcome {
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 10px;
    height: 100%;
    text-align: center;
  }
  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .welcome p {
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
    margin-top: 8px;
  }
  .keys {
    margin-top: 18px;
    font-size: 11.5px;
  }
</style>
