<script lang="ts">
  import { ms } from "../../lib/format";
  import type { HurlCall } from "../../lib/types";

  // HURL reports timings in microseconds, cumulative from the start of the call.
  let { timings }: { timings: NonNullable<HurlCall["timings"]> } = $props();

  const phases = $derived.by(() => {
    const t = timings;
    const cum = [
      { id: "dns", label: "DNS lookup", end: t.name_lookup },
      { id: "connect", label: "TCP connect", end: t.connect },
      { id: "tls", label: "TLS handshake", end: t.app_connect },
      { id: "pre", label: "Request sent", end: t.pre_transfer },
      { id: "wait", label: "Waiting (TTFB)", end: t.start_transfer },
      { id: "dl", label: "Download", end: t.total },
    ];
    let prev = 0;
    return cum.map((p) => {
      const end = Math.max(p.end, prev);
      const d = end - prev;
      const out = { ...p, start: prev, dur: d };
      prev = end;
      return out;
    });
  });
  const total = $derived(Math.max(timings.total, 1));
</script>

<div class="wf">
  {#each phases as p (p.id)}
    <div class="row">
      <span class="lbl">{p.label}</span>
      <div class="track"><div class="bar {p.id}" style:left="{(p.start / total) * 100}%" style:width="{Math.max((p.dur / total) * 100, p.dur > 0 ? 0.8 : 0)}%"></div></div>
      <span class="val mono">{ms(p.dur / 1000)}</span>
    </div>
  {/each}
  <div class="row sum"><span class="lbl">Total</span><div class="track"></div><span class="val mono">{ms(timings.total / 1000)}</span></div>
</div>

<style>
  .wf {
    display: grid;
    gap: 6px;
    padding: 12px;
  }
  .row {
    display: grid;
    grid-template-columns: 120px 1fr 70px;
    gap: 10px;
    align-items: center;
  }
  .lbl {
    color: var(--fg-muted);
    font-size: 12px;
  }
  .track {
    position: relative;
    height: 10px;
    border-radius: 5px;
    background: var(--canvas-subtle);
  }
  .bar {
    position: absolute;
    top: 0;
    bottom: 0;
    border-radius: 5px;
    background: var(--accent);
  }
  .bar.dns { background: var(--done); }
  .bar.connect { background: var(--attention); }
  .bar.tls { background: #e16f24; }
  .bar.pre { background: var(--fg-subtle); }
  .bar.wait { background: var(--success); }
  .bar.dl { background: var(--accent); }
  .val {
    text-align: right;
    font-size: 11.5px;
  }
  .sum {
    margin-top: 4px;
    font-weight: 600;
  }
</style>
