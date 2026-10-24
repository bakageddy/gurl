<script lang="ts">
  import Icon from "../Icon.svelte";
  import TextEditor from "../LazyEditor.svelte";
  import Waterfall from "./Waterfall.svelte";
  import { getResponseBody } from "../../lib/api";
  import { STATUS_TEXT } from "../../lib/hurl-schema";
  import { bytes, ms, prettyJson } from "../../lib/format";
  import { ui } from "../../lib/stores/ui.svelte";
  import type { HurlCall, HurlEntryReport } from "../../lib/types";

  interface Props {
    entry: HurlEntryReport;
    call: HurlCall;
    runId: string;
    fileIndex: number;
    entryIndex: number;
    callIndex: number;
  }
  let { entry, call, runId, fileIndex, entryIndex, callIndex }: Props = $props();

  let tab = $state<"body" | "headers" | "cookies" | "asserts" | "captures" | "timings" | "curl" | "cert">("body");
  let full = $state<string | null>(null);
  let raw = $state(false);

  const res = $derived(call.response);
  const failed = $derived(entry.asserts.filter((a) => !a.success));
  const bodyText = $derived(full ?? res.body ?? "");
  const pretty = $derived(raw ? null : prettyJson(bodyText));
  const contentType = $derived(res.headers?.find((h) => h.name.toLowerCase() === "content-type")?.value ?? "");
  const lang = $derived(contentType.includes("json") || pretty ? "json" : contentType.includes("xml") || contentType.includes("html") ? "xml" : "plain");
  const statusClass = $derived(res.status >= 500 ? "fail" : res.status >= 400 ? "warn" : res.status >= 200 && res.status < 400 ? "ok" : "");

  $effect(() => {
    // New call selected: reset the full-body override.
    call;
    full = null;
  });

  async function loadFull() {
    try {
      full = await getResponseBody(runId, fileIndex, entryIndex, callIndex);
    } catch (e) {
      ui.error(e);
    }
  }
  function copy(text: string) {
    navigator.clipboard?.writeText(text).then(() => ui.toast("Copied"));
  }
</script>

<div class="rv">
  <div class="summary">
    <span class="badge {statusClass}">{res.status} {STATUS_TEXT[res.status] ?? ""}</span>
    <span class="muted mono">{res.http_version ?? ""}</span>
    <span class="muted">{ms(call.timings ? call.timings.total / 1000 : entry.time / 1000)}</span>
    <span class="muted">{bytes(res.bodySize ?? bodyText.length)}</span>
    {#if failed.length}<span class="badge fail">{failed.length} failed assert{failed.length === 1 ? "" : "s"}</span>{/if}
  </div>

  <div class="tabs" role="tablist">
    {#each [["body", "Body"], ["headers", `Headers ${res.headers?.length ?? 0}`], ["cookies", `Cookies ${res.cookies?.length ?? 0}`], ["asserts", `Asserts ${entry.asserts.length}`], ["captures", `Captures ${entry.captures.length}`], ["timings", "Timings"], ["curl", "cURL"], ["cert", "TLS"]] as [id, label]}
      <button role="tab" class:active={tab === id} aria-selected={tab === id} onclick={() => (tab = id as typeof tab)}>
        {label}
        {#if id === "asserts" && failed.length}<span class="dot fail"></span>{/if}
      </button>
    {/each}
  </div>

  <div class="content">
    {#if tab === "body"}
      <div class="tools">
        {#if lang === "json"}
          <button class="btn sm ghost" onclick={() => (raw = !raw)}>{raw ? "Pretty" : "Raw"}</button>
        {/if}
        <button class="btn sm ghost" onclick={() => copy(pretty ?? bodyText)}><Icon name="copy" size={12} /> Copy</button>
        {#if res.bodyTruncated && full === null}
          <span class="warn-text">Showing the first {bytes(res.body?.length ?? 0)} of {bytes(res.bodySize ?? 0)}.</span>
          <button class="btn sm" onclick={loadFull}>Load full body</button>
        {/if}
      </div>
      <div class="cm">
        {#if bodyText}
          <TextEditor value={pretty ?? bodyText} {lang} readonly onchange={() => {}} />
        {:else}
          <p class="subtle pad">The response has no body.</p>
        {/if}
      </div>
    {:else if tab === "headers"}
      <table class="kv selectable">
        <tbody>
          {#each res.headers ?? [] as h}<tr><td class="k mono">{h.name}</td><td class="mono">{h.value}</td></tr>{/each}
        </tbody>
      </table>
      <h4 class="label pad">Request</h4>
      <div class="pad mono selectable req-line">{call.request.method} {call.request.url}</div>
      <table class="kv selectable">
        <tbody>
          {#each call.request.headers ?? [] as h}<tr><td class="k mono">{h.name}</td><td class="mono">{h.value}</td></tr>{/each}
        </tbody>
      </table>
    {:else if tab === "cookies"}
      {#if res.cookies?.length}
        <table class="kv selectable">
          <tbody>{#each res.cookies as c}<tr><td class="k mono">{c.name}</td><td class="mono">{c.value}</td></tr>{/each}</tbody>
        </table>
      {:else}<p class="subtle pad">No cookies were set.</p>{/if}
    {:else if tab === "asserts"}
      {#if entry.asserts.length}
        <ul class="asserts selectable">
          {#each entry.asserts as a}
            <li class:bad={!a.success}>
              <Icon name={a.success ? "check" : "x"} size={13} />
              <span>line {a.line}{#if a.message}<pre class="mono msg">{a.message}</pre>{:else} passed{/if}</span>
            </li>
          {/each}
        </ul>
      {:else}<p class="subtle pad">This request has no asserts.</p>{/if}
    {:else if tab === "captures"}
      {#if entry.captures.length}
        <table class="kv selectable">
          <tbody>{#each entry.captures as c}<tr><td class="k mono">{c.name}</td><td class="mono">{typeof c.value === "string" ? c.value : JSON.stringify(c.value)}</td></tr>{/each}</tbody>
        </table>
      {:else}<p class="subtle pad">Nothing was captured.</p>{/if}
    {:else if tab === "timings"}
      {#if call.timings}<Waterfall timings={call.timings} />{:else}<p class="subtle pad">No timing data.</p>{/if}
    {:else if tab === "curl"}
      <div class="tools"><button class="btn sm ghost" onclick={() => copy(entry.curl_cmd)}><Icon name="copy" size={12} /> Copy</button></div>
      <pre class="pad mono selectable wrap">{entry.curl_cmd}</pre>
    {:else if tab === "cert"}
      {#if res.certificate}
        <table class="kv selectable">
          <tbody>{#each Object.entries(res.certificate) as [k, v]}<tr><td class="k">{k}</td><td class="mono">{v}</td></tr>{/each}</tbody>
        </table>
      {:else}<p class="subtle pad">No TLS certificate (plain HTTP or not collected).</p>{/if}
    {/if}
  </div>
</div>

<style>
  .rv {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }
  .summary {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 12px;
    font-size: 12px;
    flex-wrap: wrap;
  }
  .tabs {
    display: flex;
    gap: 2px;
    padding: 0 8px;
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }
  .tabs button {
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fg-muted);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .tabs button.active {
    color: var(--fg);
    border-bottom-color: var(--accent);
  }
  .dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-left: 4px;
  }
  .dot.fail {
    background: var(--danger);
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }
  .tools {
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-muted);
    flex: none;
  }
  .warn-text {
    font-size: 11.5px;
    color: var(--attention);
  }
  .cm {
    flex: 1;
    min-height: 120px;
  }
  .pad {
    padding: 10px 12px;
    margin: 0;
  }
  .kv {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  .kv td {
    padding: 4px 12px;
    border-bottom: 1px solid var(--border-muted);
    vertical-align: top;
    word-break: break-all;
  }
  .kv td.k {
    width: 32%;
    color: var(--fg-muted);
    font-weight: 600;
  }
  .asserts {
    list-style: none;
    margin: 0;
    padding: 8px 12px;
    display: grid;
    gap: 6px;
  }
  .asserts li {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: var(--success);
  }
  .asserts li.bad {
    color: var(--danger);
  }
  .asserts li span {
    color: var(--fg);
  }
  .msg {
    margin: 4px 0 0;
    white-space: pre-wrap;
    color: var(--fg-muted);
  }
  .wrap {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .req-line {
    font-weight: 600;
  }
</style>
