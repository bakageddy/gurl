<script lang="ts">
  // Minimal stroke icon set (16px grid). Paths are inline so there is no icon font or runtime.
  const paths: Record<string, string> = {
    folder: "M1.5 4.5a1 1 0 0 1 1-1h3.2l1.3 1.5h6.5a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1z",
    "folder-open": "M1.5 12.5v-8a1 1 0 0 1 1-1h3.2l1.3 1.5h5.5a1 1 0 0 1 1 1V7M1.8 12.5l1.6-4.2a1 1 0 0 1 .9-.6h10.2a.6.6 0 0 1 .6.8l-1.5 4.1a1 1 0 0 1-.9.6H2.5",
    file: "M4 1.5h5l3.5 3.5v9a.5.5 0 0 1-.5.5H4a.5.5 0 0 1-.5-.5v-12A.5.5 0 0 1 4 1.5zM9 1.5V5h3.5",
    play: "M4.5 2.8v10.4a.4.4 0 0 0 .6.3l8.3-5.2a.4.4 0 0 0 0-.7L5.1 2.5a.4.4 0 0 0-.6.3z",
    stop: "M3.5 3.5h9v9h-9z",
    plus: "M8 3v10M3 8h10",
    x: "M3.5 3.5l9 9M12.5 3.5l-9 9",
    "chevron-right": "M6 3.5L10.5 8 6 12.5",
    "chevron-down": "M3.5 6L8 10.5 12.5 6",
    save: "M3 2.5h8.2L13.5 4.8V13a.5.5 0 0 1-.5.5H3a.5.5 0 0 1-.5-.5V3a.5.5 0 0 1 .5-.5zM5 2.5v3.5h5V2.5M5 13.5V9h6v4.5",
    trash: "M2.5 4h11M6 4V2.5h4V4M4 4l.6 9.2a.5.5 0 0 0 .5.5h5.8a.5.5 0 0 0 .5-.5L12 4M6.5 6.5v5M9.5 6.5v5",
    copy: "M5.5 5.5h7a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-.5.5h-7a.5.5 0 0 1-.5-.5V6a.5.5 0 0 1 .5-.5zM3 10.5v-8A.5.5 0 0 1 3.5 2h7",
    edit: "M10.5 3l2.5 2.5L5.5 13H3v-2.5zM9 4.5L11.5 7",
    search: "M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM11 11l3 3",
    gear: "M8 10.2a2.2 2.2 0 1 0 0-4.4 2.2 2.2 0 0 0 0 4.4zM8 1.5v1.7M8 12.8v1.7M1.5 8h1.7M12.8 8h1.7M3.4 3.4l1.2 1.2M11.4 11.4l1.2 1.2M12.6 3.4l-1.2 1.2M4.6 11.4l-1.2 1.2",
    sun: "M8 10.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM8 1.5v1.3M8 13.2v1.3M1.5 8h1.3M13.2 8h1.3M3.4 3.4l.9.9M11.7 11.7l.9.9M12.6 3.4l-.9.9M4.3 11.7l-.9.9",
    moon: "M13.5 9.6A5.7 5.7 0 0 1 6.4 2.5a5.7 5.7 0 1 0 7.1 7.1z",
    check: "M3 8.5l3.2 3.2L13 4.5",
    alert: "M8 2L1.8 13h12.4zM8 6.5v3M8 11.3v.2",
    download: "M8 2v8M4.8 7.2L8 10.4l3.2-3.2M2.5 13h11",
    external: "M9 2.5h4.5V7M13.5 2.5L7.5 8.5M12 9.5V13a.5.5 0 0 1-.5.5h-8A.5.5 0 0 1 3 13V5a.5.5 0 0 1 .5-.5H6.5",
    grip: "M5.5 3.5v.01M5.5 8v.01M5.5 12.5v.01M10.5 3.5v.01M10.5 8v.01M10.5 12.5v.01",
    refresh: "M13 8a5 5 0 1 1-1.5-3.6M13 2.5v3h-3",
    clock: "M8 14a6 6 0 1 0 0-12 6 6 0 0 0 0 12zM8 4.5V8l2.3 1.4",
    braces: "M5.5 2.5C4 2.5 4 3.5 4 4.5v1.3c0 1-.5 2.2-2 2.2 1.5 0 2 1.2 2 2.2v1.3c0 1 0 2 1.5 2M10.5 2.5C12 2.5 12 3.5 12 4.5v1.3c0 1 .5 2.2 2 2.2-1.5 0-2 1.2-2 2.2v1.3c0 1 0 2-1.5 2",
    list: "M5.5 4h8M5.5 8h8M5.5 12h8M2.5 4v.01M2.5 8v.01M2.5 12v.01",
    key: "M10 7a3 3 0 1 0-2.6-1.5L2 11v2.5h2.5V12h1.5v-1.5h1.5L9 9a3 3 0 0 0 1-.1",
    "arrow-up": "M8 13V3M4 7l4-4 4 4",
    "arrow-down": "M8 3v10M4 9l4 4 4-4",
    globe: "M8 14a6 6 0 1 0 0-12 6 6 0 0 0 0 12zM2 8h12M8 2c2 1.8 2.8 3.8 2.8 6S10 12.2 8 14M8 2C6 3.8 5.2 5.8 5.2 8S6 12.2 8 14",
    shield: "M8 1.8l5 1.7v4c0 3.2-2.1 5.2-5 6.7-2.9-1.5-5-3.5-5-6.7v-4z",
    dot: "M8 8v.01",
    "file-plus": "M4 1.5h5l3.5 3.5v9a.5.5 0 0 1-.5.5H4a.5.5 0 0 1-.5-.5v-12A.5.5 0 0 1 4 1.5zM8 7.5v4M6 9.5h4",
    "folder-plus": "M1.5 4.5a1 1 0 0 1 1-1h3.2l1.3 1.5h6.5a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1zM8 7.5v3.5M6.2 9.2h3.6",
    "git-merge": "M4.5 4.5v7M4.5 6.5c0 2 2 2.5 4 2.5h3M4.5 4.5a1.2 1.2 0 1 0 0-.01M4.5 11.5a1.2 1.2 0 1 0 0 .01M11.5 9a1.2 1.2 0 1 0 0 .01",
  };
  interface Props {
    name: string;
    size?: number;
    class?: string;
  }
  let { name, size = 14, class: cls = "" }: Props = $props();
</script>

<svg
  class={cls}
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.4"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={paths[name] ?? paths.dot} />
</svg>
