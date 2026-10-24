<script lang="ts">
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import JsonTreeNode from "./JsonTreeNode.svelte";
  import { nodeCount, searchTree, type JsonNode } from "../../lib/json-tree";
  import { ui } from "../../lib/stores/ui.svelte";

  interface Props {
    root: JsonNode;
    filter?: string;
    canAdd?: boolean;
    onnode: (e: MouseEvent, node: JsonNode) => void;
  }
  let { root, filter = "", canAdd = false, onnode }: Props = $props();

  const EXPAND_ALL_CAP = 5000;
  const DEFAULT_DEPTH = 1;

  const expanded = new SvelteSet<string>();
  const limits = new SvelteMap<string, number>();
  const keyOf = (n: JsonNode) => n.path.join("\u0000");

  function expandTo(depth: number) {
    const walk = (n: JsonNode) => {
      if (n.kind !== "object" && n.kind !== "array") return;
      if (n.path.length <= depth) expanded.add(keyOf(n));
      if (n.path.length < depth) n.children?.forEach(walk);
    };
    walk(root);
  }

  // New document: start with the root and its first level open.
  $effect(() => {
    root;
    expanded.clear();
    limits.clear();
    expandTo(DEFAULT_DEPTH);
  });

  const found = $derived(searchTree(root, filter.trim()));
  const matches = $derived(new Set(found.map(keyOf)));

  // Reveal matches: open every ancestor of every match (bounded by the search cap).
  $effect(() => {
    for (const n of found) {
      for (let i = 0; i < n.path.length; i++) expanded.add(n.path.slice(0, i).join("\u0000"));
    }
  });

  export function expandAll() {
    if (nodeCount(root) > EXPAND_ALL_CAP) {
      ui.toast(`Too large to expand at once (${nodeCount(root)} nodes); expand branches as you go`);
      return;
    }
    expandTo(Number.MAX_SAFE_INTEGER);
  }
  export function collapseAll() {
    expanded.clear();
    expanded.add("");
  }
</script>

<div class="tree selectable" role="tree">
  {#if filter.trim()}
    <div class="hits subtle">{found.length === 0 ? "No match" : found.length >= 500 ? "500+ matches (showing the first 500)" : `${found.length} match${found.length === 1 ? "" : "es"}`}</div>
  {/if}
  <JsonTreeNode node={root} depth={0} {expanded} {limits} {matches} {canAdd} {onnode} />
</div>

<style>
  .tree {
    padding: 4px 0 12px;
  }
  .hits {
    padding: 2px 12px 4px;
    font-size: 11.5px;
  }
</style>
