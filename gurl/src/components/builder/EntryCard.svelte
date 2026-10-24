<script lang="ts">
  import Icon from "../Icon.svelte";
  import KvTable from "./KvTable.svelte";
  import BodyEditor from "./BodyEditor.svelte";
  import MultipartEditor from "./MultipartEditor.svelte";
  import OptionsEditor from "./OptionsEditor.svelte";
  import AssertRow from "./AssertRow.svelte";
  import CaptureRow from "./CaptureRow.svelte";
  import { COMMON_STATUS, METHODS, VERSIONS } from "../../lib/hurl-schema";
  import type { EntryM } from "../../lib/types";

  interface Props {
    entry: EntryM;
    index: number;
    count: number;
    hurlPath: string | null;
    onmove: (delta: number) => void;
    onduplicate: () => void;
    ondelete: () => void;
    ontouch: () => void;
  }
  let { entry = $bindable(), index, count, hurlPath, onmove, onduplicate, ondelete, ontouch }: Props = $props();

  let open = $state(true);
  let reqTab = $state<"headers" | "query" | "body" | "form" | "multipart" | "cookies" | "auth" | "options">("headers");
  let resTab = $state<"status" | "asserts" | "captures" | "body">("asserts");

  const req = $derived(entry.request);
  const res = $derived(entry.response);

  const COMMON_HEADERS = [
    "Accept", "Accept-Encoding", "Accept-Language", "Authorization", "Cache-Control", "Content-Type",
    "Cookie", "Origin", "Referer", "User-Agent", "X-Requested-With", "X-Api-Key", "If-None-Match",
  ];

  function counts() {
    return {
      headers: req.headers.length,
      query: req.query.length,
      body: req.body ? 1 : 0,
      form: req.form.length,
      multipart: req.multipart.length,
      cookies: req.cookies.length,
      auth: req.basicAuth ? 1 : 0,
      options: req.options.length,
    };
  }
  const c = $derived(counts());

  const methodClass = $derived(
    { GET: "get", POST: "post", PUT: "put", PATCH: "patch", DELETE: "delete" }[req.method as string] ?? "other",
  );

  function addResponse() {
    entry.response = { version: "HTTP", status: "200", headers: [], captures: [], asserts: [], body: null };
    ontouch();
  }
</script>

<section class="card">
  <header class="head">
    <button class="icon-btn" onclick={() => (open = !open)} aria-label={open ? "Collapse" : "Expand"}>
      <Icon name={open ? "chevron-down" : "chevron-right"} size={13} />
    </button>
    <span class="idx subtle">#{index + 1}</span>
    <span class="method {methodClass}">{req.method}</span>
    <span class="url mono" title={req.url}>{req.url}</span>
    {#if res}<span class="badge">{res.status}</span>{/if}
    <span class="spacer"></span>
    <button class="icon-btn" title="Move up" disabled={index === 0} onclick={() => onmove(-1)}><Icon name="arrow-up" size={13} /></button>
    <button class="icon-btn" title="Move down" disabled={index === count - 1} onclick={() => onmove(1)}><Icon name="arrow-down" size={13} /></button>
    <button class="icon-btn" title="Duplicate" onclick={onduplicate}><Icon name="copy" size={13} /></button>
    <button class="icon-btn" title="Delete request" onclick={ondelete}><Icon name="trash" size={13} /></button>
  </header>

  {#if open}
    <div class="body">
      <input class="field comment" placeholder="Comment (written as # above the request)" value={entry.comments.join(" ")}
        oninput={(e) => { entry.comments = e.currentTarget.value ? [e.currentTarget.value] : []; }} />

      <div class="reqline">
        <select class="field method-select" bind:value={req.method}>
          {#each METHODS as m}<option value={m}>{m}</option>{/each}
          {#if !METHODS.includes(req.method)}<option value={req.method}>{req.method}</option>{/if}
        </select>
        <input class="field mono url-input" placeholder={"https://api.example.org/{{path}}"} bind:value={req.url} spellcheck="false" />
      </div>

      <div class="panel">
        <div class="tabs" role="tablist">
          {#each [["headers", "Headers"], ["query", "Query"], ["body", "Body"], ["form", "Form"], ["multipart", "Multipart"], ["cookies", "Cookies"], ["auth", "Auth"], ["options", "Options"]] as [id, label]}
            <button role="tab" class:active={reqTab === id} aria-selected={reqTab === id} onclick={() => (reqTab = id as typeof reqTab)}>
              {label}{#if c[id as keyof typeof c]}<span class="count">{c[id as keyof typeof c]}</span>{/if}
            </button>
          {/each}
        </div>
        <div class="tabbody">
          {#if reqTab === "headers"}
            <KvTable bind:items={req.headers} namePlaceholder="Header" valuePlaceholder="Value" suggest={COMMON_HEADERS} listId="hdr-{index}" {ontouch} addLabel="Header" />
          {:else if reqTab === "query"}
            <KvTable bind:items={req.query} namePlaceholder="Parameter" valuePlaceholder="Value" {ontouch} addLabel="Parameter" />
          {:else if reqTab === "body"}
            <BodyEditor bind:body={req.body} {hurlPath} {ontouch} />
          {:else if reqTab === "form"}
            <p class="note subtle">Sent as <code>application/x-www-form-urlencoded</code>.</p>
            <KvTable bind:items={req.form} namePlaceholder="Field" valuePlaceholder="Value" {ontouch} addLabel="Field" />
          {:else if reqTab === "multipart"}
            <MultipartEditor bind:parts={req.multipart} {hurlPath} {ontouch} />
          {:else if reqTab === "cookies"}
            <KvTable bind:items={req.cookies} namePlaceholder="Cookie" valuePlaceholder="Value" {ontouch} addLabel="Cookie" />
          {:else if reqTab === "auth"}
            {#if req.basicAuth}
              <div class="auth">
                <input class="field mono" placeholder="username" bind:value={req.basicAuth.name} />
                <input class="field mono" placeholder="password" bind:value={req.basicAuth.value} />
                <button class="icon-btn" title="Remove" onclick={() => { req.basicAuth = null; ontouch(); }}><Icon name="x" size={13} /></button>
              </div>
            {:else}
              <button class="btn sm" onclick={() => { req.basicAuth = { name: "", value: "" }; ontouch(); }}><Icon name="plus" size={12} /> Basic auth</button>
              <p class="note subtle">Digest, NTLM, Negotiate and AWS SigV4 are available under Options.</p>
            {/if}
          {:else if reqTab === "options"}
            <OptionsEditor bind:options={req.options} {ontouch} />
          {/if}
        </div>
      </div>

      {#if res}
        <div class="panel response">
          <div class="tabs" role="tablist">
            <span class="ptitle">Response</span>
            {#each [["status", "Status & headers", res.headers.length], ["asserts", "Asserts", res.asserts.length], ["captures", "Captures", res.captures.length], ["body", "Body", res.body ? 1 : 0]] as [id, label, n]}
              <button role="tab" class:active={resTab === id} aria-selected={resTab === id} onclick={() => (resTab = id as typeof resTab)}>
                {label}{#if n}<span class="count">{n}</span>{/if}
              </button>
            {/each}
            <span class="spacer"></span>
            <button class="icon-btn" title="Remove response checks" onclick={() => { entry.response = null; ontouch(); }}><Icon name="x" size={13} /></button>
          </div>
          <div class="tabbody">
            {#if resTab === "status"}
              <div class="status-line">
                <select class="field" bind:value={res.version}>
                  {#each VERSIONS as v}<option value={v.value}>{v.label}</option>{/each}
                </select>
                <input class="field mono status" list="status-{index}" placeholder="200 or *" bind:value={res.status} />
                <datalist id="status-{index}">{#each COMMON_STATUS as s}<option value={s}></option>{/each}</datalist>
              </div>
              <p class="label">Required response headers</p>
              <KvTable bind:items={res.headers} namePlaceholder="Header" valuePlaceholder="Expected value" suggest={COMMON_HEADERS} listId="rh-{index}" {ontouch} addLabel="Header" />
            {:else if resTab === "asserts"}
              <div class="stack">
                {#each res.asserts as _, i (i)}
                  <AssertRow bind:assert={res.asserts[i]} ondelete={() => { res.asserts.splice(i, 1); ontouch(); }} {ontouch} />
                {/each}
                <div class="adds">
                  <button class="btn sm" onclick={() => { res.asserts.push({ query: { kind: "jsonpath", args: '"$."' }, filters: [], not: false, func: "==", value: "" }); ontouch(); }}><Icon name="plus" size={12} /> Assert</button>
                  <button class="btn sm ghost" onclick={() => { res.asserts.push({ query: { kind: "duration", args: "" }, filters: [], not: false, func: "<", value: "1000" }); ontouch(); }}>duration &lt; 1000</button>
                  <button class="btn sm ghost" onclick={() => { res.asserts.push({ query: { kind: "header", args: '"Content-Type"' }, filters: [], not: false, func: "contains", value: '"json"' }); ontouch(); }}>header contains</button>
                </div>
              </div>
            {:else if resTab === "captures"}
              <div class="stack">
                {#each res.captures as _, i (i)}
                  <CaptureRow bind:capture={res.captures[i]} ondelete={() => { res.captures.splice(i, 1); ontouch(); }} {ontouch} />
                {/each}
                <div class="adds">
                  <button class="btn sm" onclick={() => { res.captures.push({ name: "", query: { kind: "jsonpath", args: '"$."' }, filters: [], redacted: false }); ontouch(); }}><Icon name="plus" size={12} /> Capture</button>
                </div>
              </div>
            {:else if resTab === "body"}
              <BodyEditor bind:body={res.body} {hurlPath} response {ontouch} />
            {/if}
          </div>
        </div>
      {:else}
        <button class="btn sm add-res" onclick={addResponse}><Icon name="plus" size={12} /> Add response checks (status, asserts, captures)</button>
      {/if}
    </div>
  {/if}
</section>

<style>
  .card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--canvas);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: var(--canvas-subtle);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }
  .head:not(:has(+ .body)) {
    border-radius: var(--radius-lg);
  }
  .idx {
    font-size: 12px;
  }
  .method {
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-subtle);
    color: var(--accent);
  }
  .method.get { color: var(--success); background: var(--success-subtle); }
  .method.post { color: var(--attention); background: var(--attention-subtle); }
  .method.delete { color: var(--danger); background: var(--danger-subtle); }
  .url {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg-muted);
  }
  .spacer {
    flex: 1;
  }
  .body {
    display: grid;
    gap: 10px;
    padding: 10px;
  }
  .comment {
    font-size: 12px;
    color: var(--fg-muted);
  }
  .reqline {
    display: flex;
    gap: 6px;
  }
  .method-select {
    width: 108px;
    font-weight: 600;
  }
  .url-input {
    flex: 1;
  }
  .panel {
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
  }
  .tabs {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 6px 0;
    border-bottom: 1px solid var(--border-muted);
    overflow-x: auto;
  }
  .tabs button[role="tab"] {
    height: 28px;
    padding: 0 10px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fg-muted);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .tabs button[role="tab"].active {
    color: var(--fg);
    border-bottom-color: var(--accent);
  }
  .count {
    margin-left: 5px;
    font-size: 10.5px;
    padding: 0 5px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--fg-subtle) 22%, transparent);
  }
  .ptitle {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--fg-subtle);
    padding: 0 8px;
  }
  .tabbody {
    display: grid;
    gap: 10px;
    padding: 10px;
  }
  .note {
    margin: 0;
    font-size: 12px;
  }
  .auth {
    display: grid;
    grid-template-columns: 1fr 1fr 24px;
    gap: 6px;
  }
  .status-line {
    display: flex;
    gap: 6px;
  }
  .status-line select {
    width: 130px;
  }
  .status {
    width: 110px;
  }
  .stack {
    display: grid;
    gap: 8px;
  }
  .adds {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .add-res {
    justify-self: start;
  }
  .label {
    margin: 0;
  }
</style>
