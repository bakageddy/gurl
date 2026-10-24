<script lang="ts">
  import TextEditor from "../LazyEditor.svelte";
  import DataFileInput from "./DataFileInput.svelte";
  import { BODY_KINDS } from "../../lib/hurl-schema";
  import type { BodyKind, BodyM } from "../../lib/types";

  interface Props {
    body: BodyM | null;
    hurlPath: string | null;
    /** Response bodies are assertions on the body rather than payloads. */
    response?: boolean;
    ontouch: () => void;
  }
  let { body = $bindable(), hurlPath, response = false, ontouch }: Props = $props();

  const def = $derived(BODY_KINDS.find((k) => k.kind === body?.kind));
  const multiline = $derived(
    !!body && ["multiline", "multiline-json", "multiline-xml", "multiline-raw", "graphql", "json", "xml"].includes(body.kind),
  );

  function setKind(kind: string) {
    if (kind === "none") body = null;
    else if (body) body.kind = kind as BodyKind;
    else body = { kind: kind as BodyKind, text: "" };
    ontouch();
  }
</script>

<div class="body">
  <div class="top">
    <select class="field" value={body?.kind ?? "none"} onchange={(e) => setKind(e.currentTarget.value)}>
      <option value="none">{response ? "No body assertion" : "No body"}</option>
      {#each BODY_KINDS as k}<option value={k.kind}>{k.label}</option>{/each}
    </select>
    {#if response && body}<span class="subtle">The response body must equal this value.</span>{/if}
  </div>
  {#if body}
    {#if body.kind === "file"}
      <DataFileInput bind:value={body.text} {hurlPath} onchange={ontouch} />
    {:else if multiline}
      <div class="cm">
        <TextEditor
          value={body.text}
          lang={def?.lang ?? "plain"}
          gutter={false}
          placeholder={def?.hint}
          minHeight="140px"
          onchange={(v) => {
            if (body) {
              body.text = v;
              ontouch();
            }
          }}
        />
      </div>
    {:else}
      <input class="field mono" placeholder={def?.hint} bind:value={body.text} spellcheck="false" />
    {/if}
  {/if}
</div>

<style>
  .body {
    display: grid;
    gap: 8px;
  }
  .top {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .top select {
    width: 200px;
  }
  .cm {
    height: 200px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }
</style>
