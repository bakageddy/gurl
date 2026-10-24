<script lang="ts">
  import { onMount } from "svelte";
  import { EditorState, Compartment } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, drawSelection, placeholder as cmPlaceholder } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { bracketMatching, indentOnInput } from "@codemirror/language";
  import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { linter, type Diagnostic as CmDiagnostic } from "@codemirror/lint";
  import { json } from "@codemirror/lang-json";
  import { xml } from "@codemirror/lang-xml";
  import { githubHighlight, githubTheme, hurlLanguage } from "../lib/hurl-lang";
  import type { Diagnostic } from "../lib/types";

  interface Props {
    value: string;
    onchange: (v: string) => void;
    lang?: "hurl" | "json" | "xml" | "plain";
    diagnostics?: Diagnostic[];
    readonly?: boolean;
    gutter?: boolean;
    placeholder?: string;
    minHeight?: string;
  }
  let {
    value,
    onchange,
    lang = "hurl",
    diagnostics = [],
    readonly = false,
    gutter = true,
    placeholder = "",
    minHeight,
  }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  const diagCompartment = new Compartment();

  function language() {
    if (lang === "hurl") return hurlLanguage;
    if (lang === "json") return json();
    if (lang === "xml") return xml();
    return [];
  }

  function toCm(doc: EditorState["doc"], ds: Diagnostic[]): CmDiagnostic[] {
    return ds.flatMap((d) => {
      if (d.line < 1 || d.line > doc.lines) return [];
      const start = doc.line(d.line);
      const from = Math.min(start.from + Math.max(d.column - 1, 0), start.to);
      let to = from;
      if (d.endLine >= d.line && d.endLine <= doc.lines) {
        const end = doc.line(d.endLine);
        to = Math.min(end.from + Math.max(d.endColumn - 1, 0), end.to);
      }
      if (to <= from) to = Math.min(from + 1, doc.length);
      return [{ from, to, severity: "error" as const, message: d.message.split("\n")[0] }];
    });
  }

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          gutter ? lineNumbers() : [],
          history(),
          drawSelection(),
          indentOnInput(),
          bracketMatching(),
          closeBrackets(),
          highlightActiveLine(),
          highlightSelectionMatches(),
          keymap.of([...closeBracketsKeymap, ...defaultKeymap, ...searchKeymap, ...historyKeymap, indentWithTab]),
          language(),
          githubHighlight,
          githubTheme,
          EditorView.lineWrapping,
          EditorState.readOnly.of(readonly),
          placeholder ? cmPlaceholder(placeholder) : [],
          diagCompartment.of(linter((v) => toCm(v.state.doc, diagnostics), { delay: 0 })),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onchange(u.state.doc.toString());
          }),
        ],
      }),
    });
    return () => view?.destroy();
  });

  // External value changes (e.g. builder edits regenerating the text).
  $effect(() => {
    const v = value;
    if (view && v !== view.state.doc.toString()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } });
    }
  });

  $effect(() => {
    const ds = diagnostics;
    if (view) {
      view.dispatch({
        effects: diagCompartment.reconfigure(linter((v) => toCm(v.state.doc, ds), { delay: 0 })),
      });
    }
  });
</script>

<div class="editor" bind:this={host} style:min-height={minHeight}></div>

<style>
  .editor {
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
  .editor :global(.cm-editor) {
    height: 100%;
  }
</style>
