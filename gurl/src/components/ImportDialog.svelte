<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import { importApply, importPreview, type ImportKind, type ImportResult, type ImportSource } from "../lib/api";
  import { basename, relative } from "../lib/format";
  import { tabs } from "../lib/stores/tabs.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { workspace } from "../lib/stores/workspace.svelte";

  let kind = $state<ImportKind>("curl");
  let curlText = $state("");
  let path = $state<string | null>(null);
  let oneFile = $state(false);
  let preview = $state<ImportResult | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);
  let selectedFile = $state(0);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  const dir = $derived(ui.importOpen?.dir ?? "");
  const source = $derived<ImportSource | null>(kind === "curl" ? (curlText.trim() ? { text: curlText } : null) : path ? { path } : null);

  $effect(() => {
    // Reset when (re)opened.
    if (ui.importOpen) {
      curlText = "";
      path = null;
      preview = null;
      error = null;
      selectedFile = 0;
    }
  });

  $effect(() => {
    // Debounced preview whenever the inputs change.
    const s = source;
    const o = oneFile;
    clearTimeout(timer);
    if (!s) {
      preview = null;
      error = null;
      return;
    }
    timer = setTimeout(() => void refresh(s, o), 200);
  });

  async function refresh(s: ImportSource, o: boolean) {
    const mine = ++seq;
    busy = true;
    try {
      const r = await importPreview(kind, s, o);
      if (mine !== seq) return;
      preview = r;
      error = null;
      selectedFile = 0;
    } catch (e) {
      if (mine !== seq) return;
      preview = null;
      error = typeof e === "string" ? e : String(e);
    } finally {
      if (mine === seq) busy = false;
    }
  }

  async function pick(directory: boolean) {
    const picked = await openDialog({
      directory,
      multiple: false,
      filters: directory ? undefined : kind === "postman" ? [{ name: "Postman collection", extensions: ["json"] }] : [{ name: "YAML", extensions: ["yml", "yaml"] }],
      title: directory ? "Choose the collection folder" : "Choose the collection file",
    });
    if (typeof picked === "string") path = picked;
  }

  async function doImport() {
    if (!source || !preview) return;
    busy = true;
    try {
      const written = await importApply(kind, source, oneFile, dir);
      await workspace.refresh([dir]);
      workspace.expanded[dir] = true;
      await workspace.load(dir);
      ui.importOpen = null;
      ui.toast(`Imported ${preview.files.length} scenario${preview.files.length === 1 ? "" : "s"}`);
      const first = written.find((p) => p.endsWith(".hurl"));
      if (first) void tabs.openFile(first);
    } catch (e) {
      ui.error(e);
    } finally {
      busy = false;
    }
  }

  function close() {
    ui.importOpen = null;
  }
  const shown = $derived(preview?.files[Math.min(selectedFile, Math.max(0, (preview?.files.length ?? 1) - 1))]);
</script>

{#if ui.importOpen}
  <div class="scrim" role="presentation" onclick={close}>
    <div class="dialog" role="dialog" aria-label="Import" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === "Escape" && close()}>
      <header>
        <h3>Import into <span class="mono">{workspace.root ? relative(workspace.root, dir) || basename(workspace.root) : dir}</span></h3>
        <div class="segmented">
          <button class:active={kind === "curl"} onclick={() => (kind = "curl")}>cURL</button>
          <button class:active={kind === "postman"} onclick={() => (kind = "postman")}>Postman</button>
          <button class:active={kind === "opencollection"} onclick={() => (kind = "opencollection")}>OpenCollection</button>
        </div>
      </header>

      <div class="body">
        <div class="input">
          {#if kind === "curl"}
            <textarea class="field mono" rows="7" placeholder={"Paste one or more curl commands, e.g.\ncurl -X POST https://api.example.com/login -H 'Content-Type: application/json' -d '{\"user\":\"me\"}'"} bind:value={curlText} spellcheck="false"></textarea>
            <p class="subtle">Headers, cookies, -d/-F bodies, -u, -k, -L, timeouts and more become HURL; the rest is listed as warnings.</p>
          {:else}
            <div class="pick">
              <button class="btn" onclick={() => pick(false)}><Icon name="file" size={13} /> Choose {kind === "postman" ? "collection .json" : "YAML file"}</button>
              {#if kind === "opencollection"}<button class="btn" onclick={() => pick(true)}><Icon name="folder" size={13} /> Choose collection folder</button>{/if}
              {#if path}<span class="mono subtle path">{path}</span>{/if}
            </div>
            <p class="subtle">
              {#if kind === "postman"}Postman Collection Format v2.1 (File → Export in Postman). Scripts and examples are not converted.
              {:else}OpenCollection 1.0 YAML: a single bundled file, or the folder layout Bruno writes (opencollection.yml, folder.yml, one file per request, environments/).{/if}
            </p>
          {/if}
          <label class="opt"><input type="checkbox" bind:checked={oneFile} /> One scenario per folder (requests share captures)</label>
        </div>

        <div class="preview">
          {#if busy && !preview}
            <p class="subtle">Converting…</p>
          {:else if error}
            <div class="banner error"><Icon name="alert" size={14} /><span>{error}</span></div>
          {:else if preview}
            <div class="files">
              <div class="label">{preview.files.length} file{preview.files.length === 1 ? "" : "s"}{preview.envs.length ? ` · ${preview.envs.length} gurl.env` : ""}</div>
              <ul>
                {#each preview.files as f, i}
                  <li><button class:sel={i === selectedFile} onclick={() => (selectedFile = i)}><Icon name="file" size={12} /> {f.relPath} <span class="subtle">{f.doc.entries.length}</span></button></li>
                {/each}
                {#each preview.envs as e}
                  <li class="env subtle"><Icon name="key" size={12} /> {e.relDir ? `${e.relDir}/` : ""}{e.fileName} ({e.vars.length})</li>
                {/each}
              </ul>
            </div>
            <pre class="mono text selectable">{shown?.text ?? ""}</pre>
            {#if preview.warnings.length}
              <div class="warnings">
                <div class="label">Warnings</div>
                <ul>{#each preview.warnings as w}<li>{w}</li>{/each}</ul>
              </div>
            {/if}
          {:else}
            <p class="subtle">The converted scenarios appear here.</p>
          {/if}
        </div>
      </div>

      <footer>
        <span class="spacer"></span>
        <button class="btn" onclick={close}>Cancel</button>
        <button class="btn primary" disabled={!preview || busy || !!error} onclick={doImport}><Icon name="download" size={13} /> Import {preview ? `${preview.files.length} file${preview.files.length === 1 ? "" : "s"}` : ""}</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 700;
    display: grid;
    place-items: center;
    background: rgba(1, 4, 9, 0.4);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    width: min(860px, calc(100vw - 32px));
    height: min(640px, calc(100vh - 48px));
    border-radius: 12px;
    background: var(--canvas-overlay);
    box-shadow: var(--shadow);
    border: 1px solid var(--border);
  }
  header,
  footer {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    flex: none;
  }
  header {
    border-bottom: 1px solid var(--border);
  }
  footer {
    border-top: 1px solid var(--border);
  }
  h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(260px, 38%) 1fr;
  }
  .input {
    padding: 14px 16px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow: auto;
  }
  textarea {
    width: 100%;
    resize: vertical;
    font-size: 12px;
  }
  .pick {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  .path {
    font-size: 11.5px;
    word-break: break-all;
  }
  .opt {
    font-size: 12.5px;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .preview {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: auto;
  }
  .preview > p {
    padding: 14px 16px;
  }
  .files {
    flex: none;
    max-height: 38%;
    overflow: auto;
    border-bottom: 1px solid var(--border-muted);
    padding: 8px 10px;
  }
  .files ul,
  .warnings ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .files li button {
    display: flex;
    gap: 6px;
    align-items: center;
    width: 100%;
    padding: 3px 6px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    font-size: 12px;
    text-align: left;
  }
  .files li button:hover {
    background: var(--row-hover);
  }
  .files li button.sel {
    background: var(--selection);
  }
  .files li.env {
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 3px 6px;
    font-size: 12px;
  }
  .text {
    flex: 1;
    margin: 0;
    padding: 10px 14px;
    font-size: 12px;
    overflow: auto;
    white-space: pre;
  }
  .warnings {
    flex: none;
    max-height: 30%;
    overflow: auto;
    padding: 8px 14px;
    border-top: 1px solid var(--border-muted);
    font-size: 12px;
    color: var(--attention);
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--fg-muted);
    margin-bottom: 4px;
  }
  .spacer {
    flex: 1;
  }
  .banner {
    margin: 12px;
  }
</style>
