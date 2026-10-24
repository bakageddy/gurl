<script lang="ts">
  // CodeMirror is ~400 KB; load it on first use so the shell paints immediately.
  import type { Component, ComponentProps } from "svelte";
  import type TextEditorType from "./TextEditor.svelte";

  type Props = ComponentProps<typeof TextEditorType>;
  type Commands = { openSearch?: () => void; revealLine?: (n: number) => void };
  let props: Props = $props();
  let Editor = $state<Component<Props, Commands> | null>(null);
  let inner = $state<Commands | null>(null);

  $effect.pre(() => {
    if (!Editor) import("./TextEditor.svelte").then((m) => (Editor = m.default as unknown as Component<Props, Commands>));
  });

  export function openSearch() {
    inner?.openSearch?.();
  }
  export function revealLine(n: number) {
    inner?.revealLine?.(n);
  }
</script>

{#if Editor}
  <Editor bind:this={inner} {...props} />
{:else}
  <div class="ph" style:min-height={props.minHeight}></div>
{/if}

<style>
  .ph {
    height: 100%;
    background: var(--canvas);
  }
</style>
