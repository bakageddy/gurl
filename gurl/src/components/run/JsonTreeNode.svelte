<script lang="ts">
  import JsonTreeNode from "./JsonTreeNode.svelte";
  import Icon from "../Icon.svelte";
  import { jsonPathFor } from "../../lib/assert-gen";
  import type { JsonNode } from "../../lib/json-tree";
  import type { SvelteMap, SvelteSet } from "svelte/reactivity";

  interface Props {
    node: JsonNode;
    depth: number;
    expanded: SvelteSet<string>;
    limits: SvelteMap<string, number>;
    matches: Set<string>;
    canAdd: boolean;
    onnode: (e: MouseEvent, node: JsonNode) => void;
  }
  let { node, depth, expanded, limits, matches, canAdd, onnode }: Props = $props();

  const CHUNK = 200;
  const MAX_STRING = 500;

  const key = $derived(node.path.join("\u0000"));
  const container = $derived(node.kind === "object" || node.kind === "array");
  const open = $derived(container && expanded.has(key));
  const limit = $derived(limits.get(key) ?? CHUNK);
  const shown = $derived(open ? (node.children ?? []).slice(0, limit) : []);
  const hidden = $derived(Math.max(0, node.count - limit));
  const path = $derived(jsonPathFor(node.path));
  let longOpen = $state(false);

  function toggle() {
    if (!container) return;
    if (open) expanded.delete(key);
    else expanded.add(key);
  }
  function more() {
    limits.set(key, limit + CHUNK);
  }
  function label() {
    if (node.key === null) return "";
    return typeof node.key === "number" ? String(node.key) : /^[A-Za-z_][A-Za-z0-9_]*$/.test(node.key) ? node.key : JSON.stringify(node.key);
  }
  const text = $derived(
    node.kind === "string"
      ? JSON.stringify(longOpen || node.raw.length <= MAX_STRING ? node.raw : node.raw.slice(0, MAX_STRING))
      : node.raw,
  );
</script>

<div
  class="row"
  class:match={matches.has(key)}
  style:padding-left="{8 + depth * 14}px"
  title={path}
  role="treeitem"
  aria-expanded={container ? open : undefined}
  aria-selected="false"
  tabindex="-1"
  oncontextmenu={(e) => onnode(e, node)}
>
  <button class="exp" onclick={toggle} tabindex="-1" aria-label={open ? "Collapse" : "Expand"} disabled={!container}>
    {#if container}<Icon name={open ? "chevron-down" : "chevron-right"} size={11} />{/if}
  </button>
  {#if node.key !== null}
    <span class="k" class:idx={typeof node.key === "number"}>{label()}</span><span class="colon">:</span>
  {/if}
  {#if container}
    <span class="punct">{node.kind === "array" ? "[" : "{"}</span>
    {#if !open}
      <button class="count" onclick={toggle}>{node.count} {node.kind === "array" ? (node.count === 1 ? "item" : "items") : node.count === 1 ? "key" : "keys"}</button>
      <span class="punct">{node.kind === "array" ? "]" : "}"}</span>
    {/if}
  {:else}
    <span class="v {node.kind}">{text}</span>
    {#if node.kind === "string" && node.raw.length > MAX_STRING}
      <button class="more-text" onclick={() => (longOpen = !longOpen)}>{longOpen ? "less" : `… ${node.raw.length - MAX_STRING} more chars`}</button>
    {/if}
  {/if}
  {#if canAdd}
    <button class="plus" onclick={(e) => onnode(e, node)} title="Add an assert on {path}" aria-label="Add assert"><Icon name="plus" size={11} /></button>
  {/if}
</div>
{#if open}
  {#each shown as child (child.path.join("\u0000"))}
    <JsonTreeNode node={child} depth={depth + 1} {expanded} {limits} {matches} {canAdd} {onnode} />
  {/each}
  {#if hidden > 0}
    <div class="row" style:padding-left="{8 + (depth + 1) * 14 + 18}px">
      <button class="count" onclick={more}>Show {Math.min(CHUNK, hidden)} more of {hidden}</button>
    </div>
  {/if}
  <div class="row" style:padding-left="{8 + depth * 14 + 18}px"><span class="punct">{node.kind === "array" ? "]" : "}"}</span></div>
{/if}

<style>
  .row {
    display: flex;
    align-items: baseline;
    gap: 4px;
    min-height: 20px;
    padding-right: 8px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 20px;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .row:hover {
    background: var(--row-hover);
  }
  .row.match {
    background: var(--attention-subtle);
  }
  .exp {
    flex: none;
    width: 14px;
    height: 20px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--fg-muted);
    display: inline-grid;
    place-items: center;
  }
  .exp:disabled {
    visibility: hidden;
  }
  .k {
    color: var(--syn-prop);
  }
  .k.idx {
    color: var(--fg-subtle);
  }
  .colon {
    color: var(--fg-muted);
    margin-right: 2px;
  }
  .punct {
    color: var(--fg-muted);
  }
  .v.string {
    color: var(--syn-string);
  }
  .v.number {
    color: var(--syn-number);
  }
  .v.boolean,
  .v.null {
    color: var(--syn-keyword);
  }
  .count,
  .more-text {
    border: 0;
    background: var(--canvas-inset);
    color: var(--fg-muted);
    border-radius: 10px;
    padding: 0 7px;
    font: inherit;
    font-size: 11px;
    line-height: 16px;
  }
  .count:hover,
  .more-text:hover {
    color: var(--fg);
  }
  .plus {
    flex: none;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--btn-bg);
    color: var(--fg-muted);
    display: inline-grid;
    place-items: center;
    opacity: 0;
    margin-left: auto;
    align-self: center;
  }
  .row:hover .plus,
  .plus:focus-visible {
    opacity: 1;
  }
  .plus:hover {
    color: var(--accent);
    border-color: var(--accent);
  }
</style>
