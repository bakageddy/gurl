<script lang="ts">
  // CodeMirror is ~400 KB; load it on first use so the shell paints immediately.
  import type { Component, ComponentProps } from "svelte";
  import type TextEditorType from "./TextEditor.svelte";

  type Props = ComponentProps<typeof TextEditorType>;
  let props: Props = $props();
  let Editor = $state<Component<Props> | null>(null);

  $effect.pre(() => {
    if (!Editor) import("./TextEditor.svelte").then((m) => (Editor = m.default as unknown as Component<Props>));
  });
</script>

{#if Editor}
  <Editor {...props} />
{:else}
  <div class="ph" style:min-height={props.minHeight}></div>
{/if}

<style>
  .ph {
    height: 100%;
    background: var(--canvas);
  }
</style>
