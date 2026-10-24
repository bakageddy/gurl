<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "./Icon.svelte";
  import { getDirVars, getEffectiveVars, getEnvText, setDirVars, setEnvText } from "../lib/api";
  import LazyEditor from "./LazyEditor.svelte";
  import { depthOf } from "../lib/actions";
  import { basename, debounce, relative } from "../lib/format";
  import { workspace } from "../lib/stores/workspace.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import type { EffectiveVar, EnvVar } from "../lib/types";

  let { dir }: { dir: string } = $props();

  let local = $state<EnvVar[]>([]);
  let effective = $state<EffectiveVar[]>([]);
  let loaded = $state(false);
  let saving = $state(false);
  let mode = $state<"table" | "text">("table");
  let text = $state("");

  const localNames = $derived(new Set(local.map((v) => v.name.trim()).filter(Boolean)));
  const inherited = $derived(effective.filter((v) => !localNames.has(v.name)));
  const overridesOf = (name: string) => effective.find((v) => v.name === name)?.overrides ?? [];

  async function load() {
    try {
      local = await getDirVars(dir);
      effective = await getEffectiveVars(dir);
      text = await getEnvText(dir);
      loaded = true;
    } catch (e) {
      ui.error(e);
    }
  }

  $effect(() => {
    dir;
    untrack(() => void load());
  });

  const save = debounce(async () => {
    saving = true;
    try {
      await setDirVars(dir, $state.snapshot(local));
      effective = await getEffectiveVars(dir);
      await workspace.refresh([dir.slice(0, Math.max(dir.lastIndexOf("/"), dir.lastIndexOf("\\")))]);
    } catch (e) {
      ui.error(e);
    } finally {
      saving = false;
    }
  }, 500);

  const saveText = debounce(async () => {
    saving = true;
    try {
      await setEnvText(dir, text);
      effective = await getEffectiveVars(dir);
      local = await getDirVars(dir);
      await workspace.refresh([dir.slice(0, Math.max(dir.lastIndexOf("/"), dir.lastIndexOf("\\")))]);
    } catch (e) {
      ui.error(e);
    } finally {
      saving = false;
    }
  }, 500);

  async function setMode(m: "table" | "text") {
    if (m === mode) return;
    save.cancel();
    saveText.cancel();
    // flush whatever was being edited, then reload from disk so both views agree
    if (mode === "table") await setDirVars(dir, $state.snapshot(local));
    else await setEnvText(dir, text);
    await load();
    mode = m;
  }

  function add(name = "", value = "") {
    local.push({ name, value, secret: false });
    save();
  }
  function remove(i: number) {
    local.splice(i, 1);
    save();
  }
  const rel = (p: string) => (workspace.root ? relative(workspace.root, p) || basename(p) : p);
</script>

<div class="vars">
  <header>
    <div class="segmented">
      <button class:active={mode === "table"} onclick={() => setMode("table")}>Table</button>
      <button class:active={mode === "text"} onclick={() => setMode("text")}>zurl.env</button>
    </div>
    <div>
      <h2>Variables</h2>
      <p class="muted">
        <code>{rel(dir) || basename(dir)}</code> · depth {depthOf(dir)} of {workspace.maxDepth}
        {#if saving}· saving…{/if}
      </p>
    </div>
  </header>
  <p class="hint muted">
    Defined in this folder's <code>zurl.env</code>. They apply to every scenario here and in all sub-folders; a
    sub-folder can override a name by defining it again. Reference them as <code>{"{{name}}"}</code>. Secrets are
    redacted in logs and reports.
  </p>

  {#if mode === "text"}
    <h3 class="label">zurl.env</h3>
    <p class="hint muted">
      One <code>name=value</code> per line. A <code># @secret</code> comment line marks the next variable as a secret.
    </p>
    <div class="cm">
      <LazyEditor value={text} lang="plain" placeholder={"host=https://api.example.org\n# @secret\ntoken=abc123"} onchange={(v) => { text = v; saveText(); }} />
    </div>
  {/if}

  {#if mode === "table"}
  <h3 class="label">This folder</h3>
  <div class="table" oninput={() => save()} onchange={() => save()}>
    {#if loaded && local.length === 0}<p class="subtle empty">No variables defined here yet.</p>{/if}
    {#each local as v, i (i)}
      <div class="row">
        <input class="field mono" placeholder="name" bind:value={v.name} spellcheck="false" />
        <input class="field mono" placeholder="value" type={v.secret ? "password" : "text"} bind:value={v.value} spellcheck="false" />
        <label class="secret" title="Redact this value in logs and reports"><input type="checkbox" bind:checked={v.secret} /> secret</label>
        <button class="icon-btn" title="Remove" onclick={() => remove(i)}><Icon name="trash" size={13} /></button>
        {#if overridesOf(v.name.trim()).length}
          <span class="ov subtle">overrides {overridesOf(v.name.trim()).map(rel).map((p) => p || "(root)").join(", ")}</span>
        {/if}
      </div>
    {/each}
    <button class="btn sm add" onclick={() => add()}><Icon name="plus" size={12} /> Variable</button>
  </div>

  {/if}

  {#if inherited.length}
    <h3 class="label">Inherited from parent folders</h3>
    <div class="table">
      {#each inherited as v (v.name)}
        <div class="row inh">
          <span class="mono name">{v.name}</span>
          <span class="mono val">{v.secret ? "••••••••" : v.value}</span>
          <span class="subtle src">from {rel(v.sourceDir) || "(root)"}</span>
          <button class="btn sm" onclick={() => add(v.name, v.secret ? "" : v.value)}>Override</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .vars {
    max-width: 820px;
    padding: 20px 24px;
    overflow: auto;
    height: 100%;
  }
  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }
  header {
    display: grid;
    gap: 12px;
  }
  .cm {
    height: 260px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }
  header p {
    margin: 2px 0 0;
  }
  .hint {
    font-size: 12px;
    margin: 10px 0 16px;
  }
  h3 {
    margin: 18px 0 8px;
  }
  .table {
    display: grid;
    gap: 6px;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(130px, 0.7fr) 1.4fr auto 24px;
    gap: 6px;
    align-items: center;
  }
  .row.inh {
    grid-template-columns: minmax(130px, 0.7fr) 1.4fr 1fr auto;
    padding: 4px 8px;
    border-radius: var(--radius);
    background: var(--canvas-subtle);
  }
  .name {
    font-weight: 600;
  }
  .val {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg-muted);
  }
  .src {
    font-size: 11.5px;
  }
  .secret {
    display: flex;
    gap: 4px;
    align-items: center;
    font-size: 12px;
    color: var(--fg-muted);
  }
  .ov {
    grid-column: 1 / -1;
    font-size: 11.5px;
    margin: -2px 0 2px 2px;
  }
  .add {
    justify-self: start;
  }
  .empty {
    margin: 0;
    font-size: 12px;
  }
</style>
