<script lang="ts">
  import { getContext } from "svelte";
  import Icon from "../Icon.svelte";
  import TextEditor from "../LazyEditor.svelte";
  import Waterfall from "./Waterfall.svelte";
  import JsonTree from "./JsonTree.svelte";
  import QueryBar from "./QueryBar.svelte";
  import { getResponseBody, getResponseBodyBytes, queryBody, queryText, type QueryKind, type QueryOutcome } from "../../lib/api";
  import { STATUS_TEXT } from "../../lib/hurl-schema";
  import { bytes, ms, prettyJson } from "../../lib/format";
  import { parseJsonTree, type JsonNode } from "../../lib/json-tree";
  import {
    assertText,
    captureFor,
    jsonPathFor,
    hurlString,
    suggestionsForCertificate,
    suggestionsForCookie,
    suggestionsForDuration,
    suggestionsForHeader,
    suggestionsForJson,
    suggestionsForStatus,
    type Suggestion,
  } from "../../lib/assert-gen";
  import { ASSERT_TARGET, type AssertItems, type AssertTarget } from "../../lib/assert-target";
  import { ui, type MenuItem } from "../../lib/stores/ui.svelte";
  import type { HurlCall, HurlEntryReport, QueryM } from "../../lib/types";

  interface Props {
    entry: HurlEntryReport;
    call: HurlCall;
    runId: string;
    fileIndex: number;
    entryIndex: number;
    callIndex: number;
  }
  let { entry, call, runId, fileIndex, entryIndex, callIndex }: Props = $props();

  type View = "pretty" | "raw" | "tree" | "preview";
  type Family = "json" | "xml" | "html" | "image" | "text";
  /** Last view chosen per kind of body, for the session. */
  const remembered = new Map<Family, View>();

  let tab = $state<"body" | "headers" | "cookies" | "asserts" | "captures" | "timings" | "curl" | "cert">("body");
  let full = $state<string | null>(null);
  let viewChoice = $state<View | null>(null);
  let filter = $state("");
  let showQuery = $state(false);
  let imageUrl = $state<string | null>(null);
  let editor = $state<{ openSearch: () => void } | null>(null);
  let tree = $state<{ expandAll: () => void; collapseAll: () => void } | null>(null);
  let filterInput = $state<HTMLInputElement | null>(null);

  const target = getContext<AssertTarget | undefined>(ASSERT_TARGET);
  const canAdd = $derived(target?.enabled() ?? false);

  const res = $derived(call.response);
  const failed = $derived(entry.asserts.filter((a) => !a.success));
  const bodyText = $derived(full ?? res.body ?? "");
  const contentType = $derived(res.headers?.find((h) => h.name.toLowerCase() === "content-type")?.value ?? "");
  const family = $derived<Family>(
    contentType.includes("json") || (!contentType && prettyJson(bodyText) !== null)
      ? "json"
      : contentType.includes("html")
        ? "html"
        : contentType.includes("xml")
          ? "xml"
          : contentType.startsWith("image/")
            ? "image"
            : "text",
  );
  const views = $derived<View[]>(
    family === "json" ? ["pretty", "tree", "raw"] : family === "html" ? ["raw", "preview"] : family === "image" ? ["preview", "raw"] : ["raw"],
  );
  const view = $derived<View>(viewChoice && views.includes(viewChoice) ? viewChoice : (remembered.get(family) ?? views[0]));
  const pretty = $derived(view === "pretty" ? prettyJson(bodyText) : null);
  const lang = $derived(family === "json" ? "json" : family === "xml" || family === "html" ? "xml" : "plain");
  const jsonTree = $derived(view === "tree" ? parseJsonTree(bodyText) : null);
  const queryKinds = $derived<QueryKind[]>(family === "json" ? ["jsonpath"] : family === "xml" || family === "html" ? ["xpath"] : []);
  const statusClass = $derived(res.status >= 500 ? "fail" : res.status >= 400 ? "warn" : res.status >= 200 && res.status < 400 ? "ok" : "");
  const totalMs = $derived(call.timings ? call.timings.total / 1000 : entry.time / 1000);

  $effect(() => {
    // New call selected: reset per-response state.
    call;
    full = null;
    filter = "";
    imageUrl = null;
  });

  $effect(() => {
    if (view === "preview" && family === "image" && !imageUrl) void loadImage();
  });

  function setView(v: View) {
    viewChoice = v;
    remembered.set(family, v);
  }

  async function loadFull() {
    try {
      full = await getResponseBody(runId, fileIndex, entryIndex, callIndex);
    } catch (e) {
      ui.error(e);
    }
  }
  async function loadImage() {
    try {
      const b = await getResponseBodyBytes(runId, fileIndex, entryIndex, callIndex);
      imageUrl = `data:${b.contentType ?? "application/octet-stream"};base64,${b.base64}`;
    } catch (e) {
      ui.error(e);
    }
  }
  function copy(text: string) {
    navigator.clipboard?.writeText(text).then(() => ui.toast("Copied"));
  }
  function search() {
    if (view === "tree") filterInput?.focus();
    else editor?.openSearch();
  }

  /** The body kept in memory for this run; history runs fall back to the preview text. */
  async function runQuery(kind: QueryKind, expr: string): Promise<QueryOutcome> {
    try {
      return await queryBody(runId, fileIndex, entryIndex, callIndex, kind, expr);
    } catch {
      return queryText(bodyText, contentType || null, kind, expr);
    }
  }

  async function add(items: AssertItems) {
    if (!target) return;
    try {
      await target.add(entry.index, items);
    } catch (e) {
      ui.error(e);
    }
  }

  /** The context menu for anything assertable: suggestions, capture, copy. */
  function openMenu(e: MouseEvent, suggestions: Suggestion[], extra: { captureQuery?: QueryM; copy?: Array<[string, string]> } = {}) {
    const items: MenuItem[] = [];
    if (!canAdd) items.push({ label: target?.reason?.() ?? "Can't add asserts here", action: () => {}, disabled: true });
    for (const s of suggestions) items.push({ label: assertText(s.assert), action: () => void add({ asserts: [s.assert] }), disabled: !canAdd });
    if (extra.captureQuery) {
      const q = extra.captureQuery;
      items.push({ label: "", action: () => {}, separator: true });
      items.push({
        label: "Capture as variable…",
        disabled: !canAdd,
        action: async () => {
          const name = await ui.ask({ title: "Capture as variable", label: "Variable name", value: suggestName(q), confirmLabel: "Add capture" });
          if (name?.trim()) void add({ captures: [captureFor(q, name.trim())] });
        },
      });
    }
    if (extra.copy?.length) {
      items.push({ label: "", action: () => {}, separator: true });
      for (const [label, text] of extra.copy) items.push({ label, action: () => copy(text) });
    }
    ui.openMenu(e, items);
  }

  function suggestName(q: QueryM) {
    const m = /([A-Za-z_][A-Za-z0-9_]*)['"\]]*$/.exec(q.args);
    return m ? m[1] : q.kind;
  }

  function onJsonNode(e: MouseEvent, node: JsonNode) {
    const path = jsonPathFor(node.path);
    const suggestions = suggestionsForJson(node);
    openMenu(e, suggestions, {
      captureQuery: { kind: "jsonpath", args: hurlString(path) },
      copy: [
        ["Copy JSONPath", path],
        ["Copy value", node.kind === "string" ? node.raw : node.children ? JSON.stringify(valueOf(node)) : node.raw],
        ...(suggestions[0] ? ([["Copy as assert line", assertText(suggestions[0].assert)]] as Array<[string, string]>) : []),
      ],
    });
  }
  function valueOf(node: JsonNode): unknown {
    switch (node.kind) {
      case "object":
        return Object.fromEntries((node.children ?? []).map((c) => [c.key, valueOf(c)]));
      case "array":
        return (node.children ?? []).map(valueOf);
      case "string":
        return node.raw;
      case "number":
        return Number(node.raw);
      case "boolean":
        return node.raw === "true";
      default:
        return null;
    }
  }
  const onHeader = (e: MouseEvent, name: string, value: string) =>
    openMenu(e, suggestionsForHeader(name, value), { captureQuery: { kind: "header", args: hurlString(name) }, copy: [["Copy value", value]] });
  const onCookie = (e: MouseEvent, name: string, value: string) =>
    openMenu(e, suggestionsForCookie(name, value), { captureQuery: { kind: "cookie", args: hurlString(name) }, copy: [["Copy value", value]] });
  const onDuration = (e: MouseEvent) => openMenu(e, suggestionsForDuration(totalMs));
  const onStatus = (e: MouseEvent) => openMenu(e, suggestionsForStatus(res.status));
  const onCert = (e: MouseEvent, field: string, value: string) =>
    openMenu(e, suggestionsForCertificate(field, value), { captureQuery: { kind: "certificate", args: hurlString(field) } });

  /** HTML preview: no scripts, no network, no same-origin; the document can only style itself. */
  const previewDoc = $derived(
    family === "html"
      ? `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data:;">` + bodyText
      : "",
  );
</script>

<div class="rv">
  <div class="summary">
    <button class="badge {statusClass} clickable" onclick={onStatus} title="Assert on the status">{res.status} {STATUS_TEXT[res.status] ?? ""}</button>
    <span class="muted mono">{res.http_version ?? ""}</span>
    <button class="muted clickable plain" onclick={onDuration} title="Assert on the duration">{ms(totalMs)}</button>
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
        {#if views.length > 1}
          <div class="segmented sm">
            {#each views as v}
              <button class:active={view === v} onclick={() => setView(v)}>{v === "pretty" ? "Pretty" : v === "raw" ? "Raw" : v === "tree" ? "Tree" : "Preview"}</button>
            {/each}
          </div>
        {/if}
        {#if view === "tree"}
          <input class="field sm filter" type="search" placeholder="Filter keys and values" bind:value={filter} bind:this={filterInput} />
          <button class="btn sm ghost" onclick={() => tree?.expandAll()} title="Expand all">Expand</button>
          <button class="btn sm ghost" onclick={() => tree?.collapseAll()} title="Collapse all">Collapse</button>
        {:else if view !== "preview"}
          <button class="btn sm ghost" onclick={search} title="Search in the body (Ctrl/Cmd+F inside the body)"><Icon name="search" size={12} /> Search</button>
        {/if}
        {#if queryKinds.length}
          <button class="btn sm ghost" class:active={showQuery} onclick={() => (showQuery = !showQuery)} title="Try a {queryKinds[0]} query and turn it into an assert"><Icon name="braces" size={12} /> Query</button>
        {/if}
        <span class="spacer"></span>
        <button class="btn sm ghost" onclick={() => copy(pretty ?? bodyText)}><Icon name="copy" size={12} /> Copy</button>
        {#if res.bodyTruncated && full === null}
          <span class="warn-text">Showing the first {bytes(res.body?.length ?? 0)} of {bytes(res.bodySize ?? 0)}.</span>
          <button class="btn sm" onclick={loadFull}>Load full body</button>
        {/if}
      </div>
      {#if showQuery && queryKinds.length}
        <QueryBar kinds={queryKinds} run={runQuery} {canAdd} onadd={(items) => void add(items)} />
      {/if}
      <div class="cm">
        {#if !bodyText && family !== "image"}
          <p class="subtle pad">The response has no body.</p>
        {:else if view === "tree"}
          {#if jsonTree}
            <JsonTree bind:this={tree} root={jsonTree} {filter} {canAdd} onnode={onJsonNode} />
          {:else}
            <p class="subtle pad">The body isn't valid JSON; use Raw.</p>
          {/if}
        {:else if view === "preview"}
          {#if family === "image"}
            {#if imageUrl}<div class="img-box"><img src={imageUrl} alt="Response body" /></div>{:else}<p class="subtle pad">Loading…</p>{/if}
          {:else}
            <iframe class="preview" title="HTML preview" sandbox="" referrerpolicy="no-referrer" srcdoc={previewDoc}></iframe>
          {/if}
        {:else}
          <TextEditor bind:this={editor} value={pretty ?? bodyText} {lang} readonly onchange={() => {}} />
        {/if}
      </div>
    {:else if tab === "headers"}
      <table class="kv selectable">
        <tbody>
          {#each res.headers ?? [] as h}
            <tr class="hov">
              <td class="k mono">{h.name}</td>
              <td class="mono">{h.value}</td>
              <td class="act"><button class="plus" onclick={(e) => onHeader(e, h.name, h.value)} title="Assert on this header" aria-label="Add assert"><Icon name="plus" size={11} /></button></td>
            </tr>
          {/each}
        </tbody>
      </table>
      <h4 class="label pad">Request</h4>
      <div class="pad mono selectable req-line">{call.request.method} {call.request.url}</div>
      <table class="kv selectable">
        <tbody>
          {#each call.request.headers ?? [] as h}<tr><td class="k mono">{h.name}</td><td class="mono" colspan="2">{h.value}</td></tr>{/each}
        </tbody>
      </table>
    {:else if tab === "cookies"}
      {#if res.cookies?.length}
        <table class="kv selectable">
          <tbody>
            {#each res.cookies as c}
              <tr class="hov">
                <td class="k mono">{c.name}</td>
                <td class="mono">{c.value}</td>
                <td class="act"><button class="plus" onclick={(e) => onCookie(e, c.name, c.value)} title="Assert on this cookie" aria-label="Add assert"><Icon name="plus" size={11} /></button></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}<p class="subtle pad">No cookies were set.</p>{/if}
    {:else if tab === "asserts"}
      {#if entry.asserts.length}
        <ul class="asserts selectable">
          {#each entry.asserts as a}
            <li class:bad={!a.success}>
              <Icon name={a.success ? "check" : "x"} size={13} />
              <div class="a-main">
                <code class="cond mono">{a.source ?? `line ${a.line}`}</code>
                {#if a.message}<pre class="mono msg">{a.message}</pre>{/if}
              </div>
              <span class="ln subtle">line {a.line}</span>
            </li>
          {/each}
        </ul>
      {:else}<p class="subtle pad">This request has no asserts. Click a value in the body, a header or a cookie to add one.</p>{/if}
    {:else if tab === "captures"}
      {#if entry.captures.length}
        <table class="kv selectable">
          <tbody>{#each entry.captures as c}<tr><td class="k mono">{c.name}</td><td class="mono" colspan="2">{typeof c.value === "string" ? c.value : JSON.stringify(c.value)}</td></tr>{/each}</tbody>
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
          <tbody>
            {#each Object.entries(res.certificate) as [k, v]}
              <tr class="hov">
                <td class="k">{k}</td>
                <td class="mono">{v}</td>
                <td class="act"><button class="plus" onclick={(e) => onCert(e, k, String(v ?? ""))} title="Assert on this field" aria-label="Add assert"><Icon name="plus" size={11} /></button></td>
              </tr>
            {/each}
          </tbody>
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
  .clickable {
    cursor: pointer;
  }
  .plain {
    border: 0;
    background: transparent;
    padding: 0;
    font: inherit;
  }
  .plain:hover,
  .badge.clickable:hover {
    text-decoration: underline;
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
    flex-wrap: wrap;
  }
  .tools .btn.active {
    background: var(--accent-subtle);
    color: var(--accent);
  }
  .segmented.sm button {
    height: 22px;
    padding: 0 8px;
    font-size: 11.5px;
  }
  .filter {
    width: 180px;
    height: 24px;
    font-size: 12px;
  }
  .spacer {
    flex: 1;
  }
  .warn-text {
    font-size: 11.5px;
    color: var(--attention);
  }
  .cm {
    flex: 1;
    min-height: 120px;
    overflow: auto;
  }
  .preview {
    width: 100%;
    height: 100%;
    min-height: 240px;
    border: 0;
    background: #fff;
  }
  .img-box {
    padding: 12px;
  }
  .img-box img {
    max-width: 100%;
    background:
      linear-gradient(45deg, var(--canvas-inset) 25%, transparent 25%, transparent 75%, var(--canvas-inset) 75%),
      linear-gradient(45deg, var(--canvas-inset) 25%, transparent 25%, transparent 75%, var(--canvas-inset) 75%);
    background-size: 16px 16px;
    background-position:
      0 0,
      8px 8px;
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
  .kv td.act {
    width: 30px;
    padding: 2px 6px;
    text-align: right;
  }
  .plus {
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
  }
  .hov:hover .plus,
  .plus:focus-visible {
    opacity: 1;
  }
  .plus:hover {
    color: var(--accent);
    border-color: var(--accent);
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
  .a-main {
    flex: 1;
    min-width: 0;
    color: var(--fg);
  }
  .cond {
    word-break: break-all;
  }
  .ln {
    font-size: 11px;
    flex: none;
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
