<script lang="ts">
  // A line diff (expected → actual) with long runs of unchanged lines folded.
  import type { DiffRow } from "../../lib/types";

  let { rows, context = 3 }: { rows: DiffRow[]; context?: number } = $props();

  type Block = { kind: "rows"; rows: Array<DiffRow & { n: number }> } | { kind: "fold"; count: number; rows: Array<DiffRow & { n: number }> };

  let openFolds = $state(new Set<number>());

  const blocks = $derived.by(() => {
    const numbered = rows.map((r, n) => ({ ...r, n }));
    const out: Block[] = [];
    let i = 0;
    while (i < numbered.length) {
      if (numbered[i].op !== "eq") {
        const start = i;
        while (i < numbered.length && numbered[i].op !== "eq") i++;
        out.push({ kind: "rows", rows: numbered.slice(start, i) });
        continue;
      }
      const start = i;
      while (i < numbered.length && numbered[i].op === "eq") i++;
      const run = numbered.slice(start, i);
      const first = start === 0;
      const last = i === numbered.length;
      const keepHead = first ? 0 : context;
      const keepTail = last ? 0 : context;
      if (run.length > keepHead + keepTail + 2) {
        if (keepHead) out.push({ kind: "rows", rows: run.slice(0, keepHead) });
        out.push({ kind: "fold", count: run.length - keepHead - keepTail, rows: run.slice(keepHead, run.length - keepTail) });
        if (keepTail) out.push({ kind: "rows", rows: run.slice(run.length - keepTail) });
      } else {
        out.push({ kind: "rows", rows: run });
      }
    }
    return out;
  });

  const sign = (op: DiffRow["op"]) => (op === "del" ? "−" : op === "ins" ? "+" : " ");
</script>

<div class="diff mono selectable">
  <div class="legend">
    <span class="del">− expected</span>
    <span class="ins">+ actual</span>
  </div>
  {#each blocks as b, bi}
    {#if b.kind === "fold" && !openFolds.has(bi)}
      <button class="fold" onclick={() => (openFolds = new Set(openFolds).add(bi))}>⋯ {b.count} unchanged line{b.count === 1 ? "" : "s"}</button>
    {:else}
      {#each b.rows as r (r.n)}
        {#if r.op === "cut"}
          <div class="row cut">⋯ diff truncated</div>
        {:else}
          <div class="row {r.op}">
            <span class="sign">{sign(r.op)}</span>
            <span class="text">{r.text}{#if r.noEol}<span class="noeol" title="No newline at end">⏎̸</span>{/if}</span>
          </div>
        {/if}
      {/each}
    {/if}
  {/each}
</div>

<style>
  .diff {
    margin: 6px 0 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: auto;
    font-size: 12px;
    line-height: 1.5;
    background: var(--canvas);
    color: var(--fg);
  }
  .legend {
    display: flex;
    gap: 12px;
    padding: 3px 10px;
    font-size: 11px;
    border-bottom: 1px solid var(--border-muted);
    background: var(--canvas-subtle);
  }
  .legend .del {
    color: var(--danger);
  }
  .legend .ins {
    color: var(--success);
  }
  .row {
    display: flex;
    white-space: pre;
  }
  .sign {
    flex: none;
    width: 22px;
    text-align: center;
    user-select: none;
    color: var(--fg-muted);
  }
  .text {
    flex: 1;
    padding-right: 10px;
  }
  .row.del {
    background: color-mix(in srgb, var(--danger) 14%, transparent);
  }
  .row.del .sign {
    color: var(--danger);
  }
  .row.ins {
    background: color-mix(in srgb, var(--success) 14%, transparent);
  }
  .row.ins .sign {
    color: var(--success);
  }
  .row.cut,
  .fold {
    color: var(--fg-muted);
    padding: 2px 10px;
    font-size: 11px;
  }
  .fold {
    display: block;
    width: 100%;
    border: 0;
    background: var(--canvas-subtle);
    text-align: left;
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  .fold:hover {
    color: var(--fg);
  }
  .noeol {
    color: var(--danger);
    margin-left: 4px;
    font-size: 11px;
  }
</style>
