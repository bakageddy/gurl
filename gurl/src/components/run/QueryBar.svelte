<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "../Icon.svelte";
  import JsonTree from "./JsonTree.svelte";
  import { assertText, captureFor, suggestionsForQuery } from "../../lib/assert-gen";
  import { parseJsonTree } from "../../lib/json-tree";
  import type { QueryKind, QueryOutcome } from "../../lib/api";
  import { ui } from "../../lib/stores/ui.svelte";
  import type { AssertM, CaptureM } from "../../lib/types";

  interface Props {
    kinds: QueryKind[];
    run: (kind: QueryKind, expr: string) => Promise<QueryOutcome>;
    canAdd: boolean;
    onadd: (items: { asserts?: AssertM[]; captures?: CaptureM[] }) => void;
  }
  let { kinds, run, canAdd, onadd }: Props = $props();

  let kind = $state<QueryKind>(untrack(() => kinds[0] ?? "jsonpath"));
  let expr = $state("");
  let outcome = $state<QueryOutcome | null>(null);
  let busy = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  $effect(() => {
    if (!kinds.includes(kind)) kind = kinds[0] ?? "jsonpath";
  });

  const placeholder = $derived(kind === "jsonpath" ? '$.items[0].id   $.items[?@.id > 1].name   $..name' : 'string(//h1)   count(//item)   //a/@href');

  function schedule() {
    clearTimeout(timer);
    const e = expr.trim();
    if (!e) {
      outcome = null;
      return;
    }
    timer = setTimeout(() => void evaluate(e), 150);
  }

  async function evaluate(e: string) {
    const mine = ++seq;
    busy = true;
    try {
      const out = await run(kind, e);
      if (mine === seq) outcome = out;
    } catch (err) {
      if (mine === seq) outcome = { kind: "error", value: null, text: "", count: 0, error: String(err) };
    } finally {
      if (mine === seq) busy = false;
    }
  }

  const tree = $derived(outcome && outcome.text && (outcome.kind === "single" || outcome.kind === "list") ? parseJsonTree(outcome.text) : null);
  const suggestions = $derived(outcome && outcome.kind !== "error" ? suggestionsForQuery(kind, expr.trim(), outcome.kind, outcome.value, outcome.count) : []);
  const scalar = $derived(
    !outcome ? "" : outcome.kind === "none" ? "no match" : outcome.kind === "nodeset" ? `${outcome.count} node${outcome.count === 1 ? "" : "s"}` : outcome.kind === "list" ? `${outcome.count} items` : outcome.text,
  );

  function add(a: AssertM) {
    onadd({ asserts: [a] });
  }
  function more(e: MouseEvent) {
    ui.openMenu(
      e,
      suggestions.map((s) => ({ label: assertText(s.assert), action: () => add(s.assert) })),
    );
  }
  async function capture() {
    const name = await ui.ask({ title: "Capture as variable", label: "Variable name", value: "value", confirmLabel: "Add capture" });
    if (!name?.trim()) return;
    const q = { kind, args: suggestions[0]?.assert.query.args ?? JSON.stringify(expr.trim()) };
    onadd({ captures: [captureFor(q, name.trim())] });
  }
  function copyLine() {
    const s = suggestions[0];
    if (s) navigator.clipboard?.writeText(assertText(s.assert)).then(() => ui.toast("Copied"));
  }
</script>

<div class="qb">
  <div class="line">
    {#if kinds.length > 1}
      <select class="field kind" bind:value={kind} onchange={schedule}>
        {#each kinds as k}<option value={k}>{k}</option>{/each}
      </select>
    {:else}
      <span class="kind-label mono">{kind}</span>
    {/if}
    <input class="field mono expr" type="text" spellcheck="false" {placeholder} bind:value={expr} oninput={schedule} onkeydown={(e) => e.key === "Enter" && suggestions[0] && canAdd && add(suggestions[0].assert)} />
    {#if busy}<span class="spin"></span>{/if}
  </div>
  {#if outcome}
    <div class="result">
      {#if outcome.kind === "error"}
        <div class="err">{outcome.error}</div>
      {:else}
        <div class="head">
          <span class="badge">{outcome.kind}</span>
          <span class="mono scalar">{scalar}</span>
          <span class="spacer"></span>
          {#if suggestions[0]}
            <code class="mono preview" title="The assert that would be added">{assertText(suggestions[0].assert)}</code>
            <button class="btn sm primary" disabled={!canAdd} onclick={() => add(suggestions[0].assert)} title={canAdd ? "Add this assert to the request (Enter)" : "Can't add here"}><Icon name="plus" size={11} /> Add assert</button>
            {#if suggestions.length > 1}<button class="btn sm" onclick={more}>More…</button>{/if}
            <button class="btn sm" disabled={!canAdd} onclick={capture} title="Capture this query into a variable">Capture…</button>
            <button class="icon-btn" onclick={copyLine} title="Copy assert line"><Icon name="copy" size={12} /></button>
          {/if}
        </div>
        {#if tree && (tree.kind === "object" || tree.kind === "array")}
          <div class="tree-box"><JsonTree root={tree} onnode={() => {}} /></div>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .qb {
    border-bottom: 1px solid var(--border-muted);
    flex: none;
  }
  .line {
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 6px 10px;
  }
  .kind {
    width: 92px;
  }
  .kind-label {
    font-size: 11.5px;
    color: var(--fg-muted);
  }
  .expr {
    flex: 1;
    min-width: 0;
  }
  .result {
    padding: 0 10px 6px;
  }
  .head {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 12px;
  }
  .scalar {
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .preview {
    font-size: 11.5px;
    color: var(--fg-muted);
    max-width: 34%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .err {
    color: var(--danger);
    font-size: 12px;
  }
  .tree-box {
    max-height: 220px;
    overflow: auto;
    margin-top: 4px;
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    background: var(--canvas);
  }
  .spacer {
    flex: 1;
  }
  .spin {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
